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

use aoide_core::{usecases::media::tracker::DirTrackingStatus, util::url::BaseUrl};

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::{source::Repo as MediaSourceRepo, tracker::Repo as MediaTrackerRepo},
    track::EntityRepo,
};

use crate::media::tracker::resolve_path_prefix_from_base_url;

use super::*;

pub fn purge_by_media_source_path_predicates<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    path_predicates: Vec<StringPredicate>,
) -> RepoResult<usize>
where
    Repo: EntityRepo + MediaSourceRepo,
{
    let mut total_purged_tracks = 0;
    for path_predicate in path_predicates {
        let purged_tracks = repo.purge_tracks_by_media_source_media_source_path_predicate(
            collection_id,
            path_predicate.borrow(),
        )?;
        let _purged_media_sources =
            repo.purge_media_sources_by_path_predicate(collection_id, path_predicate.borrow())?;
        debug_assert_eq!(purged_tracks, _purged_media_sources);
        total_purged_tracks += purged_tracks;
    }
    Ok(total_purged_tracks)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PurgeByUntrackedMediaSourcesSummary {
    pub untracked_directories: usize,
    pub purged_tracks: usize,
}

pub fn purge_by_untracked_media_sources<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    source_path_resolver: &VirtualFilePathResolver,
    root_url: Option<&BaseUrl>,
    untrack_orphaned_directories: bool,
) -> Result<PurgeByUntrackedMediaSourcesSummary>
where
    Repo: EntityRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let root_path_prefix = root_url
        .map(|url| resolve_path_prefix_from_base_url(source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let mut summary = PurgeByUntrackedMediaSourcesSummary::default();
    if untrack_orphaned_directories {
        summary.untracked_directories += repo.media_tracker_untrack(
            collection_id,
            &root_path_prefix,
            Some(DirTrackingStatus::Orphaned),
        )?;
    };
    let untracked_media_sources =
        repo.media_tracker_find_untracked_sources(collection_id, &root_path_prefix)?;
    summary.purged_tracks += repo.purge_tracks_by_media_sources(&untracked_media_sources)?;
    Ok(summary)
}
