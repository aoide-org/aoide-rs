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

mod uc {
    pub use crate::usecases::media::tracker::query_status::*;
}

use aoide_core::{entity::EntityUid, util::url::BaseUrl};
use aoide_core_serde::usecases::media::tracker::Status;

use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,
}

pub type RequestBody = Params;
pub type ResponseBody = Status;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody { root_url } = request_body;
    let root_url = root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
    uc::query_status(&pooled_connection, collection_uid, root_url.as_ref())
        .map(Into::into)
        .map_err(Into::into)
}
