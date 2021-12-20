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

use uc::collection::{create_entity, store_created_entity};

use super::*;

pub fn create(connection: &SqliteConnection, created_collection: Collection) -> Result<Entity> {
    let new_entity = create_entity(created_collection)?;
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        store_created_entity(&db, &new_entity).map_err(transaction_error)
    })?;
    Ok(new_entity)
}
