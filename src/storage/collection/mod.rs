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
        // TODO: Generate UID
        let uid = "lvVzOxqS7mS48EGgnaDYCIZ309nzRM9Op0TTRv5B02Y".to_string();
        let header = EntityHeader::with_uid(uid);
        let entity = CollectionEntity { header, name: name.into() };
        // TODO: Store entity
        entity
    }

    pub fn update_entity(entity: &CollectionEntity) -> EntityRevision {
        let next_revision = entity.header.revision().next();
        // TODO: Store entity
        next_revision
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn store_entity() {
  }
}
