// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use domain::collection::*;
use domain::entity::*;

///////////////////////////////////////////////////////////////////////
/// CollectionRepository
///////////////////////////////////////////////////////////////////////

pub struct CollectionRepository;

impl CollectionRepository {
    pub fn create_entity<S: Into<String>>(name: S) -> CollectionEntity {
        let entity = CollectionEntity::with_name(name);
        // TODO: Store entity
        entity
    }

    pub fn update_entity(entity: &mut CollectionEntity) -> EntityRevision {
        let next_revision = entity.header().revision().next();
        // TODO: Store entity
        entity.update_revision(next_revision);
        next_revision
    }

    pub fn remove_entity(_uid: &EntityUid) {
        // TODO: Delete entity from storage
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn create_entity() {
      let entity = CollectionRepository::create_entity("Test Collection");
      println!("Created entity: {:?}", entity);
      assert!(entity.is_valid());
  }

  #[test]
  fn update_entity() {
      let mut entity = CollectionRepository::create_entity("Test Collection");
      println!("Created entity: {:?}", entity);
      assert!(entity.is_valid());
      let initial_revision = entity.header().revision();
      entity.set_name("Renamed Collection");
      let updated_revision = CollectionRepository::update_entity(&mut entity);
      println!("Updated entity: {:?}", entity);
      assert!(initial_revision < updated_revision);
      assert!(entity.header().revision() == updated_revision);
  }
}
