// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    collection::EntityHeader as CollectionEntityHeader, prelude::*, util::clock::OffsetDateTimeMs,
    Collection, CollectionEntity, CollectionUid,
};
use aoide_core_api::collection::{EntityWithSummary, LoadScope};
use aoide_repo::collection::EntityRepo;

use super::*;

pub mod vfs;

#[derive(Debug)]
pub struct ValidatedInput(Collection);

pub fn validate_input(collection: Collection) -> InputResult<ValidatedInput> {
    if let Err(err) = collection.validate() {
        return Err(anyhow::anyhow!("invalid collection input: {err:?}").into());
    }
    Ok(ValidatedInput(collection))
}

pub fn create_entity(new_collection: Collection) -> Result<CollectionEntity> {
    let ValidatedInput(collection) = validate_input(new_collection)?;
    let header = CollectionEntityHeader::initial_random();
    let entity = CollectionEntity::new(header, collection);
    Ok(entity)
}

pub fn store_created_entity(repo: &mut impl EntityRepo, entity: &CollectionEntity) -> Result<()> {
    let created_at = OffsetDateTimeMs::now_utc();
    repo.insert_collection_entity(created_at, entity)?;
    Ok(())
}

pub fn update_entity(
    hdr: CollectionEntityHeader,
    modified_collection: Collection,
) -> Result<CollectionEntity> {
    let ValidatedInput(collection) = validate_input(modified_collection)?;
    let next_hdr = hdr
        .next_rev()
        .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
    let updated_entity = CollectionEntity::new(next_hdr, collection);
    Ok(updated_entity)
}

pub fn store_updated_entity(repo: &mut impl EntityRepo, entity: &CollectionEntity) -> Result<()> {
    let updated_at = OffsetDateTimeMs::now_utc();
    repo.update_collection_entity_revision(updated_at, entity)?;
    Ok(())
}

pub fn load_one(
    repo: &mut impl EntityRepo,
    collection_uid: &CollectionUid,
    scope: LoadScope,
) -> Result<EntityWithSummary> {
    let id = repo.resolve_collection_id(collection_uid)?;
    let (record_hdr, entity) = repo.load_collection_entity(id)?;
    let summary = match scope {
        LoadScope::Entity => None,
        LoadScope::EntityWithSummary => Some(repo.load_collection_summary(record_hdr.id)?),
    };
    Ok(EntityWithSummary { entity, summary })
}

pub fn purge(repo: &mut impl EntityRepo, collection_uid: &CollectionUid) -> Result<()> {
    let id = repo.resolve_collection_id(collection_uid)?;
    repo.purge_collection_entity(id).map_err(Into::into)
}
