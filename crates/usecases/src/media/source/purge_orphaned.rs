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

use aoide_core_api::media::source::purge_orphaned::{Outcome, Params};

use aoide_repo::{
    collection::EntityRepo as CollectionRepo, media::source::Repo as MediaSourceRepo,
    track::EntityRepo,
};

use crate::collection::vfs::RepoContext;

use super::*;

/// Purge orphaned media sources that don't belong to any track
pub fn purge_orphaned<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    params: &Params,
) -> Result<Outcome>
where
    Repo: CollectionRepo + EntityRepo + MediaSourceRepo,
{
    let Params { root_url } = params;
    let collection_ctx = RepoContext::resolve(repo, collection_uid, root_url.as_ref())?;
    let collection_id = collection_ctx.record_id;
    let root_path_prefix = collection_ctx.root_path_prefix_str(root_url.as_ref());
    let purged = if let Some(root_path_prefix) = root_path_prefix {
        let root_path_predicate = StringPredicateBorrowed::Prefix(root_path_prefix);
        repo.purge_orphaned_media_sources_by_path_predicate(collection_id, root_path_predicate)
    } else {
        repo.purge_orphaned_media_sources(collection_id)
    }?;
    let root_url = collection_ctx.source_path.vfs.map(|vfs| vfs.root_url);
    let outcome = Outcome { root_url, purged };
    Ok(outcome)
}
