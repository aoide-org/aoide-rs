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

use aoide_usecases_sqlite::collection::update as uc;

use super::*;

pub type QueryParams = EntityRevQueryParams;

pub type RequestBody = Collection;

pub type ResponseBody = Entity;

pub fn handle_request(
    connection: &SqliteConnection,
    uid: EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let EntityRevQueryParams { rev } = query_params;
    let updated_entity_with_current_rev = _inner::Entity::try_new(
        _inner::EntityHeader {
            uid,
            rev: rev.into(),
        },
        request_body,
    )?;
    connection
        .transaction::<_, Error, _>(|| {
            uc::update(connection, updated_entity_with_current_rev).map_err(Into::into)
        })
        .map(Into::into)
}
