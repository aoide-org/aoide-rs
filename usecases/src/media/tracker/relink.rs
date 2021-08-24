// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;

use crate::tracks::find_duplicate::{self, find_duplicate};

use aoide_core::{
    media::Source as MediaSource,
    track::{Entity, Track},
    usecases::tracks::search::{ConditionFilter, SearchFilter, SortField, SortOrder},
};

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::{source::Repo as MediaSourceRepo, tracker::Repo as MediaTrackerRepo},
    track::EntityRepo as TrackRepo,
};

use std::sync::atomic::{AtomicBool, Ordering};

pub type FindCandidateParams = find_duplicate::Params;

/// Relink a moved track.
///
/// Replace the track referenced by `old_media_source_path` with the replacement
/// referenced by `new_media_source_path`. Afterwards the replacement track is
/// deleted which requires that is not yet referenced in the collection,
/// e.g. as a playlist entry.
///
/// The `collected_at` timestamp of the old track is preserved while all
/// other properties are copied from the replacement track.
///
/// The media tracker is also updated, i.e. it will reference the updated
/// old media source instead of the new media source that is removed.
fn relink_moved_track_by_media_source_path<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    old_media_source_path: &str,
    new_media_source_path: &str,
) -> RepoResult<()>
where
    Repo: TrackRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let (old_source_id, old_header, old_entity) =
        repo.load_track_entity_by_media_source_path(collection_id, old_media_source_path)?;
    let (new_source_id, new_header, new_entity) =
        repo.load_track_entity_by_media_source_path(collection_id, new_media_source_path)?;
    let updated_track = Track {
        media_source: MediaSource {
            // Preserve the collected_at field from the old source
            collected_at: old_entity.body.media_source.collected_at,
            ..new_entity.body.media_source
        },
        ..new_entity.body
    };
    // Relink the sources in the media tracker
    repo.media_tracker_relink_source(old_source_id, new_source_id)?;
    // Delete the soon obsolete track and source records to prevent
    // constraint violations during the update. This only works as
    // long as the track is not referenced elsewhere, e.g. playlists!
    repo.delete_track_entity(new_header.id)?;
    repo.delete_media_source(new_source_id)?;
    // Finish with updating the old track
    if updated_track != old_entity.body {
        let updated_at = DateTime::now_utc();
        if old_entity.body.media_source != updated_track.media_source {
            repo.update_media_source(old_source_id, updated_at, &updated_track.media_source)?;
            debug_assert_eq!(
                updated_track.media_source,
                repo.load_media_source_by_path(collection_id, new_media_source_path)?
                    .1
            );
        }
        let updated_entity = Entity::new(old_entity.hdr, updated_track);
        repo.update_track_entity(old_header.id, updated_at, old_source_id, &updated_entity)?;
        debug_assert_eq!(
            updated_entity.body,
            repo.load_track_entity_by_media_source_path(collection_id, new_media_source_path)?
                .2
                .body
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RelocatedMediaSource {
    pub old_path: String,
    pub new_path: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Progress {
    total: usize,
    relinked: usize,
    skipped: usize,
}

impl Progress {
    fn new(total: usize) -> Self {
        Self {
            total,
            relinked: 0,
            skipped: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.total
    }

    pub fn relinked(&self) -> usize {
        self.relinked
    }

    pub fn skipped(&self) -> usize {
        self.skipped
    }

    pub fn finished(&self) -> usize {
        self.relinked + self.skipped
    }

    pub fn remaining(&self) -> usize {
        debug_assert!(self.finished() <= self.total);
        self.total - self.finished()
    }
}

pub fn relink_tracks_with_untracked_media_sources<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    mut find_candidate_params: FindCandidateParams,
    progress_fn: &mut impl FnMut(&Progress),
    abort_flag: &AtomicBool,
) -> RepoResult<Vec<RelocatedMediaSource>>
where
    Repo: TrackRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let source_untracked_filter = SearchFilter::Condition(ConditionFilter::SourceUntracked);
    let ordering = vec![SortOrder {
        field: SortField::SourceCollectedAt,
        direction: SortDirection::Descending,
    }];
    let mut lost_tracks = Vec::new();
    repo.search_collected_tracks(
        collection_id,
        &Default::default(),
        Some(source_untracked_filter),
        ordering,
        &mut lost_tracks,
    )?;
    // Only consider tracks with a tracked media source
    find_candidate_params.search_flags |= find_duplicate::SearchFlags::SOURCE_TRACKED;
    let mut progress = Progress::new(lost_tracks.len());
    let mut relinked_media_sources = Vec::with_capacity(lost_tracks.len());
    for (old_header, old_entity) in lost_tracks {
        if abort_flag.load(Ordering::Relaxed) {
            tracing::info!("Aborting");
            return Ok(relinked_media_sources);
        }
        progress_fn(&progress);
        let old_media_source_path = old_entity.body.media_source.path.clone();
        let candidates = find_duplicate(
            repo,
            collection_id,
            old_header.id,
            old_entity.body,
            &find_candidate_params,
        )?;
        let new_media_source_path = match candidates.len() {
            0 => {
                tracing::warn!("No successor found for {}", old_media_source_path);
                progress.skipped += 1;
                continue;
            }
            1 => candidates
                .into_iter()
                .map(|(_, entity)| entity.body.media_source.path)
                .next()
                .expect("single URI"),
            _ => {
                tracing::warn!(
                    "Found {} potential successors for {}: {:?}",
                    candidates.len(),
                    old_media_source_path,
                    candidates
                );
                progress.skipped += 1;
                continue;
            }
        };
        tracing::info!(
            "Found successor for {}: {}",
            old_media_source_path,
            new_media_source_path
        );
        // TODO: Avoid reloading of both old/new entities by their path
        relink_moved_track_by_media_source_path(
            repo,
            collection_id,
            &old_media_source_path,
            &new_media_source_path,
        )?;
        relinked_media_sources.push(RelocatedMediaSource {
            old_path: old_media_source_path.into(),
            new_path: new_media_source_path.into(),
        });
        progress.relinked += 1;
    }
    Ok(relinked_media_sources)
}
