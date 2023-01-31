// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::collection::{EntityWithSummary, LoadScope};

use aoide_core::{
    collection::{Collection, Entity, EntityHeader as CollectionHeader, EntityUid},
    prelude::*,
    util::clock::DateTime,
};

use aoide_repo::collection::EntityRepo;

use super::*;

pub mod vfs;

#[derive(Debug)]
pub struct ValidatedInput(Collection);

pub fn validate_input(collection: Collection) -> InputResult<ValidatedInput> {
    if let Err(err) = collection.validate() {
        return Err(anyhow::anyhow!("Invalid collection input: {err:?}").into());
    }
    Ok(ValidatedInput(collection))
}

pub fn create_entity(new_collection: Collection) -> Result<Entity> {
    let ValidatedInput(collection) = validate_input(new_collection)?;
    let header = CollectionHeader::initial_random();
    let entity = Entity::new(header, collection);
    Ok(entity)
}

pub fn store_created_entity(repo: &mut impl EntityRepo, entity: &Entity) -> Result<()> {
    let created_at = DateTime::now_utc();
    repo.insert_collection_entity(created_at, entity)?;
    Ok(())
}

pub fn update_entity(hdr: CollectionHeader, modified_collection: Collection) -> Result<Entity> {
    let ValidatedInput(collection) = validate_input(modified_collection)?;
    let next_hdr = hdr
        .next_rev()
        .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
    let updated_entity = Entity::new(next_hdr, collection);
    Ok(updated_entity)
}

pub fn store_updated_entity(repo: &mut impl EntityRepo, entity: &Entity) -> Result<()> {
    let updated_at = DateTime::now_utc();
    repo.update_collection_entity_revision(updated_at, entity)?;
    Ok(())
}

pub fn load_one(
    repo: &mut impl EntityRepo,
    entity_uid: &EntityUid,
    scope: LoadScope,
) -> Result<EntityWithSummary> {
    let id = repo.resolve_collection_id(entity_uid)?;
    let (record_hdr, entity) = repo.load_collection_entity(id)?;
    let summary = match scope {
        LoadScope::Entity => None,
        LoadScope::EntityWithSummary => Some(repo.load_collection_summary(record_hdr.id)?),
    };
    Ok(EntityWithSummary { entity, summary })
}

pub fn purge(repo: &mut impl EntityRepo, entity_uid: &EntityUid) -> Result<()> {
    let id = repo.resolve_collection_id(entity_uid)?;
    repo.purge_collection_entity(id).map_err(Into::into)
}
