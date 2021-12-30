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
use aoide_core_api::media::tracker::untrack::{Outcome, Params, Summary};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::tracker::Repo as MediaTrackerRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

pub fn untrack<Repo>(repo: &Repo, collection_uid: &EntityUid, params: &Params) -> Result<Outcome>
where
    Repo: CollectionRepo + MediaTrackerRepo,
{
    let Params { root_url, status } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, Some(root_url))?;
    let vfs_ctx = if let Some(vfs_ctx) = &collection_ctx.vfs {
        vfs_ctx
    } else {
        return Err(anyhow::anyhow!("Not supported by non-VFS collections").into());
    };
    let collection_id = collection_ctx.record_id;
    let untracked = repo.media_tracker_untrack(collection_id, &vfs_ctx.root_path, *status)?;
    let summary = Summary { untracked };
    let root_url = collection_ctx
        .vfs
        .map(|vfs_context| vfs_context.root_url)
        .unwrap();
    Ok(Outcome { root_url, summary })
}
