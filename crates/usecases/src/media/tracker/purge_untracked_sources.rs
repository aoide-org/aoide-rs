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

use aoide_core::{entity::EntityUid, util::url::BaseUrl};
use aoide_core_api::media::tracker::{
    purge_untracked_sources::{Outcome, Params, Summary},
    DirTrackingStatus,
};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::{source::Repo as MediaSourceRepo, tracker::Repo as MediaTrackerRepo},
    track::EntityRepo,
};

use crate::{
    collection::resolve_collection_id_for_virtual_file_path,
    media::tracker::resolve_path_prefix_from_base_url,
};

use super::*;

pub fn purge_untracked_sources<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + EntityRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let (collection_id, source_path_resolver) =
        resolve_collection_id_for_virtual_file_path(repo, collection_uid, None)?;
    let Params {
        root_url,
        untrack_orphaned_directories,
    } = params;
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(&source_path_resolver, url))
        .transpose()?
        .unwrap_or_default();
    let root_url = source_path_resolver
        .resolve_url_from_path(&root_path_prefix)
        .map_err(anyhow::Error::from)?;
    let root_url = BaseUrl::new(root_url);
    let mut summary = Summary::default();
    if untrack_orphaned_directories.unwrap_or(false) {
        summary.untracked_directories += repo.media_tracker_untrack(
            collection_id,
            &root_path_prefix,
            Some(DirTrackingStatus::Orphaned),
        )?;
    };
    // Purge orphaned media sources that don't belong to any track
    summary.purged_media_sources += if root_path_prefix.is_empty() {
        repo.purge_orphaned_media_sources(collection_id)
    } else {
        let root_path_predicate = StringPredicateBorrowed::Prefix(&root_path_prefix);
        repo.purge_orphaned_media_sources_by_path_predicate(collection_id, root_path_predicate)
    }?;
    // Find and purge the remaining untracked media sources
    // TODO: Purging could be optimized with a combined, mutable
    // database query that affects both repositories. But this would
    // then require that either media sources know about the media tracker
    // or that the media tracker mutates the storage of media sources.
    for record_id in repo.media_tracker_find_untracked_sources(collection_id, &root_path_prefix)? {
        repo.purge_media_source(record_id)?;
        summary.purged_media_sources += 1;
    }
    let outcome = Outcome { root_url, summary };
    Ok(outcome)
}
