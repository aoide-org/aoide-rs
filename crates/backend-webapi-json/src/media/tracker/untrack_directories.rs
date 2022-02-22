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

use aoide_core::entity::EntityUid;

use super::*;

mod uc {
    pub use aoide_usecases_sqlite::media::tracker::untrack_directories::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::untrack_directories::Params;

pub type ResponseBody = aoide_core_api_json::media::tracker::untrack_directories::Outcome;

#[tracing::instrument(
    name = "Untracking media sources",
    skip(
        connection,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    connection
        .transaction::<_, Error, _>(|| {
            uc::untrack_directories(connection, collection_uid, &params).map_err(Into::into)
        })
        .map(Into::into)
}