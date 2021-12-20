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

use semval::Validate as _;

use aoide_core::{
    collection::{Collection, Entity},
    entity::{EntityHeader, EntityUid},
    util::{clock::DateTime, url::BaseUrl},
};

use aoide_media::resolver::VirtualFilePathResolver;

use aoide_repo::collection::{EntityRepo, RecordId as CollectionId};

use super::*;

pub fn load_virtual_file_path_resolver<Repo>(
    repo: &Repo,
    collection_id: CollectionId,
    override_root_url: Option<BaseUrl>,
) -> Result<VirtualFilePathResolver>
where
    Repo: EntityRepo,
{
    let (_, entity) = repo.load_collection_entity(collection_id)?;
    let (path_kind, root_url) = entity.body.media_source_config.source_path.into();
    let root_url = if let Some(root_url) = root_url {
        root_url
    } else {
        return Err(anyhow::anyhow!("Unsupported media source path kind: {:?}", path_kind).into());
    };
    let resolver = VirtualFilePathResolver::with_root_url(override_root_url.unwrap_or(root_url));
    Ok(resolver)
}

pub fn resolve_collection_id_for_virtual_file_path<Repo>(
    repo: &Repo,
    collection_uid: &EntityUid,
    override_root_url: Option<BaseUrl>,
) -> Result<(CollectionId, VirtualFilePathResolver)>
where
    Repo: EntityRepo,
{
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    let resolver = load_virtual_file_path_resolver(repo, collection_id, override_root_url)?;
    Ok((collection_id, resolver))
}

#[derive(Debug)]
pub struct ValidatedInput(Collection);

pub fn validate_input(collection: Collection) -> InputResult<ValidatedInput> {
    if let Err(err) = collection.validate() {
        return Err(anyhow::anyhow!("Invalid collection input: {:?}", err).into());
    }
    Ok(ValidatedInput(collection))
}

pub fn create_entity(collection: Collection) -> Result<Entity> {
    let ValidatedInput(collection) = validate_input(collection)?;
    let header = EntityHeader::initial_random();
    let entity = Entity::new(header, collection);
    Ok(entity)
}

pub fn store_created_entity(repo: &impl EntityRepo, entity: &Entity) -> Result<()> {
    let created_at = DateTime::now_utc();
    repo.insert_collection_entity(created_at, entity)?;
    Ok(())
}

pub fn update_entity(hdr: EntityHeader, collection: Collection) -> Result<Entity> {
    let ValidatedInput(collection) = validate_input(collection)?;
    let next_hdr = hdr
        .next_rev()
        .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
    let updated_entity = Entity::new(next_hdr, collection);
    Ok(updated_entity)
}
