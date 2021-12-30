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

use aoide_core::entity::EntityUid;

use aoide_core_api::media::tracker::{
    purge_untracked_sources::{Outcome, Params, Summary},
    DirTrackingStatus,
};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo,
    media::{source::Repo as MediaSourceRepo, tracker::Repo as MediaTrackerRepo},
    track::EntityRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn purge_untracked_sources<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + EntityRepo + MediaSourceRepo + MediaTrackerRepo,
{
    let Params {
        root_url,
        untrack_orphaned_directories,
    } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!("Not supported by non-VFS collections").into());
    };
    let collection_id = collection_ctx.record_id;
    let mut summary = Summary::default();
    if untrack_orphaned_directories.unwrap_or(false) {
        summary.untracked_directories += repo.media_tracker_untrack(
            collection_id,
            &vfs_ctx.root_path,
            Some(DirTrackingStatus::Orphaned),
        )?;
    };
    // Purge orphaned media sources that don't belong to any track
    summary.purged_sources += if vfs_ctx.root_path.is_empty() {
        repo.purge_untracked_media_sources(collection_id)
    } else {
        let root_path_predicate = StringPredicateBorrowed::Prefix(&vfs_ctx.root_path);
        repo.purge_untracked_media_sources_by_path_predicate(collection_id, root_path_predicate)
    }?;
    let root_url = collection_ctx
        .vfs
        .map(|vfs_context| vfs_context.root_url)
        .unwrap();
    let outcome = Outcome { root_url, summary };
    Ok(outcome)
}
