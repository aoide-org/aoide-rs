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

use aoide_core::util::clock::DateTime;
use uc::playlist::update_entity;

use super::*;

pub fn update(
    connection: &SqliteConnection,
    updated_entity_with_current_rev: Entity,
) -> Result<Entity> {
    let (hdr, body) = updated_entity_with_current_rev.into();
    let entity = update_entity(hdr, body)?;
    let updated_at = DateTime::now_utc();
    let repo = RepoConnection::new(connection);
    repo.update_playlist_entity_revision(updated_at, &entity)?;
    Ok(entity)
}
