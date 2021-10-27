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

use crate::usecases::collection::create as uc;

use super::*;

pub type RequestBody = Collection;

pub type ResponseBody = Entity;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    uc::create(&pooled_connection, request_body.try_into()?)
        .map(Into::into)
        .map_err(Into::into)
}
