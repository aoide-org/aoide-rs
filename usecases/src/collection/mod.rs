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

use aoide_core::{
    entity::EntityUid,
    media::{resolver::LocalFileResolver, SourcePathKind},
};
use aoide_repo::collection::{EntityRepo, RecordId as CollectionId};

pub fn resolve_local_file_collection_id<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
) -> Result<(CollectionId, LocalFileResolver)>
where
    Repo: EntityRepo,
{
    // TODO: Load collection entity by UID with a single query
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let (_, entity) = repo.load_collection_entity(collection_id)?;
    if entity.body.media_source_config.path_kind != SourcePathKind::LocalFile {
        return Err(anyhow::anyhow!(
            "Unsupported media source path kind: {:?}",
            entity.body.media_source_config.path_kind
        )
        .into());
    }
    let source_path_resolver = if let Some(base_url) = entity.body.media_source_config.base_url {
        LocalFileResolver::with_base_url(base_url)
    } else {
        LocalFileResolver::new()
    };
    Ok((collection_id, source_path_resolver))
}
