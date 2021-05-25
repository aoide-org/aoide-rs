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

use crate::usecases::playlists::update as uc;

///////////////////////////////////////////////////////////////////////

pub type RequestBody = Playlist;

pub type ResponseBody = Entity;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    uid: EntityUid,
    query_params: EntityRevQueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let EntityRevQueryParams { rev } = query_params;
    let updated_entity_with_current_rev = _core::Entity::new(
        _core::EntityHeader {
            uid,
            rev: rev.into(),
        },
        request_body,
    );
    uc::update(&pooled_connection, updated_entity_with_current_rev)
        .map(Into::into)
        .map_err(Into::into)
}
