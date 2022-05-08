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

use uc::playlist::{create_entity, store_created_entity};

use super::*;

pub fn create(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    new_playlist: Playlist,
) -> Result<Entity> {
    let created_entity = create_entity(new_playlist)?;
    let repo = RepoConnection::new(connection);
    store_created_entity(&repo, collection_uid, &created_entity)?;
    Ok(created_entity)
}
