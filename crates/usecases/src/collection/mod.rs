// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    collection::{Collection, Entity, EntityHeader as CollectionHeader},
    util::clock::DateTime,
};

use aoide_repo::collection::EntityRepo;

use super::*;

pub mod vfs;

#[derive(Debug)]
pub struct ValidatedInput(Collection);

pub fn validate_input(collection: Collection) -> InputResult<ValidatedInput> {
    if let Err(err) = collection.validate() {
        return Err(anyhow::anyhow!("Invalid collection input: {:?}", err).into());
    }
    Ok(ValidatedInput(collection))
}

pub fn create_entity(new_collection: Collection) -> Result<Entity> {
    let ValidatedInput(collection) = validate_input(new_collection)?;
    let header = CollectionHeader::initial_random();
    let entity = Entity::new(header, collection);
    Ok(entity)
}

pub fn store_created_entity(repo: &impl EntityRepo, entity: &Entity) -> Result<()> {
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
