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

use aoide_core::util::clock::DateTime;

///////////////////////////////////////////////////////////////////////

pub fn create(connection: &SqliteConnection, new_collection: Collection) -> Result<Entity> {
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr, new_collection);
    let created_at = DateTime::now_utc();
    let db = RepoConnection::new(connection);
    Ok(db.transaction::<_, DieselRepoError, _>(|| {
        db.insert_collection_entity(created_at, &entity)?;
        Ok(entity)
    })?)
}
