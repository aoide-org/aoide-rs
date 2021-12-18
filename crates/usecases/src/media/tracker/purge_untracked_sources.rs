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

use aoide_core::util::url::BaseUrl;
use aoide_core_ext::media::tracker::{
    purge_untracked_sources::{Outcome, Params, Summary},
    DirTrackingStatus,
};

use aoide_repo::{
    collection::RecordId as CollectionId,
    media::{source::Repo as MediaSourceRepo, tracker::Repo as MediaTrackerRepo},
    track::EntityRepo,
};

use crate::media::tracker::resolve_path_prefix_from_base_url;

use super::*;

pub fn purge_untracked_sources<Repo>(
    repo: &Repo,
    source_path_resolver: &VirtualFilePathResolver,
    collection_id: CollectionId,
    params: &Params,
) -> Result<Outcome>
where
    Repo: EntityRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let Params {
        root_url,
        untrack_orphaned_directories,
    } = params;
    let root_path_prefix = root_url
        .as_ref()
        .map(|url| resolve_path_prefix_from_base_url(source_path_resolver, url))
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
    repo.media_tracker_find_untracked_sources(collection_id, &root_path_prefix)?;
    summary.purged_media_sources += if root_path_prefix.is_empty() {
        repo.purge_orphaned_media_sources(collection_id)
    } else {
        let root_path_predicate = StringPredicateBorrowed::Prefix(&root_path_prefix);
        repo.purge_orphaned_media_sources_by_path_predicate(collection_id, root_path_predicate)
    }?;
    let outcome = Outcome { root_url, summary };
    Ok(outcome)
}