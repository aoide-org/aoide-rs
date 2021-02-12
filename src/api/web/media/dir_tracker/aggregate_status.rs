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
    pub use crate::usecases::media::*;
}

use aoide_core::entity::EntityUid;

use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    pub root_url: Url,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackingStatusAggregated {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

impl From<uc::TrackingStatusAggregated> for TrackingStatusAggregated {
    fn from(from: uc::TrackingStatusAggregated) -> Self {
        let uc::TrackingStatusAggregated {
            current,
            outdated,
            added,
            modified,
            orphaned,
        } = from;
        Self {
            current,
            outdated,
            added,
            modified,
            orphaned,
        }
    }
}

pub type RequestBody = Params;
pub type ResponseBody = TrackingStatusAggregated;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody { root_url } = request_body;
    Ok(
        uc::digest_directories_aggregate_status(&pooled_connection, collection_uid, &root_url)
            .map(Into::into)?,
    )
}
