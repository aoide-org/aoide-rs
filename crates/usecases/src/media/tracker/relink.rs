// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::sync::atomic::{AtomicBool, Ordering};

use aoide_core::{
    media::Source as MediaSource,
    track::{Entity, EntityBody, Track},
};

use aoide_core_api::track::search::{ConditionFilter, SearchFilter, SortField, SortOrder};

use aoide_repo::{
    collection::{EntityRepo as CollectionRepo, RecordId as CollectionId},
    media::{
        source::{CollectionRepo as MediaSourceCollectionRepo, Repo as MediaSourceRepo},
        tracker::Repo as MediaTrackerRepo,
    },
    track::{CollectionRepo as TrackCollectionRepo, EntityRepo as TrackRepo},
};

use crate::track::find_duplicates::{self, find_duplicates};

use super::*;

pub type FindCandidateParams = find_duplicates::Params;

/// Relink a moved track.
///
/// Replace the track referenced by `old_content_link_path` with the replacement
/// referenced by `new_content_link_path`. Afterwards the replacement track is
/// deleted which requires that is not yet referenced in the collection,
/// e.g. as a playlist entry.
///
/// The `collected_at` timestamp of the old track is preserved while all
/// other properties are copied from the replacement track.
///
/// The media tracker is also updated, i.e. it will reference the updated
/// old media source instead of the new media source that is removed.
fn relink_moved_track_by_content_link_path<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    old_content_link_path: &str,
    new_content_link_path: &str,
) -> RepoResult<()>
where
    Repo: TrackRepo + TrackCollectionRepo + MediaSourceRepo + MediaSourceCollectionRepo,
{
    let (old_source_id, old_header, old_entity) =
        repo.load_track_entity_by_media_source_content_path(collection_id, old_content_link_path)?;
    let (new_source_id, new_header, new_entity) =
        repo.load_track_entity_by_media_source_content_path(collection_id, new_content_link_path)?;
    let updated_track = Track {
        media_source: MediaSource {
            // Preserve the collected_at field from the old source
            collected_at: old_entity.body.track.media_source.collected_at,
            ..new_entity.raw.body.track.media_source
        },
        ..new_entity.raw.body.track
    };
    // Relink the sources in the media tracker
    repo.purge_media_source(new_source_id)?;
    // Purging the media source must also recursively purge
    // the associated track!
    debug_assert!(matches!(
        repo.load_track_entity(new_header.id),
        Err(RepoError::NotFound)
    ));
    // Finish with updating the old track
    if updated_track != old_entity.body.track {
        let updated_at = DateTime::now_local_or_utc();
        let updated_entity_body = EntityBody {
            track: updated_track,
            updated_at,
            last_synchronized_rev: old_entity.body.last_synchronized_rev,
        };
        if old_entity.body.track.media_source != updated_entity_body.track.media_source {
            repo.update_media_source(
                old_source_id,
                updated_at,
                &updated_entity_body.track.media_source,
            )?;
            debug_assert_eq!(
                updated_entity_body.track.media_source,
                repo.load_media_source_by_path(collection_id, new_content_link_path)?
                    .1
            );
        }
        let updated_entity = Entity::new(old_entity.raw.hdr, updated_entity_body);
        repo.update_track_entity(old_header.id, old_source_id, &updated_entity)?;
        debug_assert_eq!(
            updated_entity.body,
            repo.load_track_entity_by_media_source_content_path(
                collection_id,
                new_content_link_path
            )?
            .2
            .body
        );
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelocatedMediaSource {
    pub old_path: String,
    pub new_path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    #[must_use]
    pub fn total(&self) -> usize {
        self.total
    }

    #[must_use]
    pub fn relinked(&self) -> usize {
        self.relinked
    }

    #[must_use]
    pub fn skipped(&self) -> usize {
        self.skipped
    }

    #[must_use]
    pub fn finished(&self) -> usize {
        self.relinked + self.skipped
    }

    #[must_use]
    pub fn remaining(&self) -> usize {
        debug_assert!(self.finished() <= self.total);
        self.total - self.finished()
    }
}

pub fn relink_tracks_with_untracked_media_sources<Repo, ReportProgressFn: FnMut(&Progress)>(
    repo: &Repo,
    collection_uid: &CollectionUid,
    mut find_candidate_params: FindCandidateParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> RepoResult<Vec<RelocatedMediaSource>>
where
    Repo: CollectionRepo
        + TrackRepo
        + TrackCollectionRepo
        + MediaSourceRepo
        + MediaSourceCollectionRepo
        + MediaTrackerRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let source_untracked_filter = SearchFilter::Condition(ConditionFilter::SourceUntracked);
    let ordering = vec![SortOrder {
        field: SortField::CollectedAt,
        direction: SortDirection::Descending,
    }];
    let mut lost_tracks = Vec::new();
    repo.search_tracks(
        collection_id,
        &Default::default(),
        Some(source_untracked_filter),
        ordering,
        &mut lost_tracks,
    )?;
    // Only consider tracks with a tracked media source
    find_candidate_params.search_flags |= find_duplicates::SearchFlags::SOURCE_TRACKED;
    let mut progress = Progress::new(lost_tracks.len());
    let mut relinked_media_sources = Vec::with_capacity(lost_tracks.len());
    for (old_header, old_entity) in lost_tracks {
        if abort_flag.load(Ordering::Relaxed) {
            log::info!("Aborting");
            return Ok(relinked_media_sources);
        }
        report_progress_fn(&progress);
        let old_content_link_path = old_entity.body.track.media_source.content_link.path.clone();
        let candidates = find_duplicates(
            repo,
            collection_id,
            old_header.id,
            old_entity.raw.body.track,
            &find_candidate_params,
        )?;
        let new_content_link_path = match candidates.len() {
            0 => {
                log::warn!("No successor found for {old_content_link_path}");
                progress.skipped += 1;
                continue;
            }
            1 => candidates
                .into_iter()
                .map(|(_, entity)| entity.raw.body.track.media_source.content_link.path)
                .next()
                .expect("single URI"),
            _ => {
                log::warn!(
                    "Found {num_candidates} potential successor(s) for {old_content_link_path}: {candidates:?}",
                    num_candidates = candidates.len(),
                );
                progress.skipped += 1;
                continue;
            }
        };
        log::info!("Found successor for {old_content_link_path}: {new_content_link_path}");
        // TODO: Avoid reloading of both old/new entities by their path
        relink_moved_track_by_content_link_path(
            repo,
            collection_id,
            &old_content_link_path,
            &new_content_link_path,
        )?;
        relinked_media_sources.push(RelocatedMediaSource {
            old_path: old_content_link_path.into(),
            new_path: new_content_link_path.into(),
        });
        progress.relinked += 1;
    }
    Ok(relinked_media_sources)
}
