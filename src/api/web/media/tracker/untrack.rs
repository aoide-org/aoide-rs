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

use aoide_core::{entity::EntityUid, util::url::BaseUrl};

use aoide_core_ext_serde::media::tracker::untrack::{Outcome, Params};

use super::*;

mod uc {
    pub use crate::usecases::media::tracker::untrack::*;
}

pub type RequestBody = Params;

pub type ResponseBody = Outcome;

#[tracing::instrument(
    name = "Untracking media sources",
    skip(
        pooled_connection,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody { root_url, status } = request_body;
    let root_url = BaseUrl::try_autocomplete_from(root_url)
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
    uc::untrack(
        &pooled_connection,
        collection_uid,
        root_url,
        status.map(Into::into),
    )
    .map(Into::into)
    .map_err(Into::into)
}
