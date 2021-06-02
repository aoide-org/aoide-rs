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

use aoide_core::{entity::EntityUid, media::SourcePathKind};
use aoide_media::resolver::VirtualFilePathResolver;
use aoide_repo::collection::{EntityRepo, RecordId as CollectionId};

use url::Url;

pub fn load_virtual_file_path_resolver<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    override_root_url: Option<Url>,
) -> Result<VirtualFilePathResolver>
where
    Repo: EntityRepo,
{
    let (_, entity) = repo.load_collection_entity(collection_id)?;
    if entity.body.media_source_config.path_kind != SourcePathKind::VirtualFilePath {
        return Err(anyhow::anyhow!(
            "Unsupported media source path kind: {:?}",
            entity.body.media_source_config.path_kind
        )
        .into());
    }
    let resolver = if let Some(root_url) = entity.body.media_source_config.root_url {
        VirtualFilePathResolver::with_root_url(override_root_url.unwrap_or(root_url))
    } else {
        VirtualFilePathResolver::new()
    };
    Ok(resolver)
}

pub fn resolve_collection_id_for_virtual_file_path<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    override_root_url: Option<Url>,
) -> Result<(CollectionId, VirtualFilePathResolver)>
where
    Repo: EntityRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let resolver = load_virtual_file_path_resolver(repo, collection_id, override_root_url)?;
    Ok((collection_id, resolver))
}
