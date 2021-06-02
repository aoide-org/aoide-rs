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
    pub use crate::usecases::media::tracker::untrack::*;
}

use aoide_core::entity::EntityUid;

use aoide_core_serde::usecases::media::tracker::untrack::Outcome;
use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum DirTrackingStatus {
    Current,
    Outdated,
    Added,
    Modified,
    Orphaned,
}

impl From<DirTrackingStatus> for uc::DirTrackingStatus {
    fn from(from: DirTrackingStatus) -> Self {
        use DirTrackingStatus::*;
        match from {
            Current => Self::Current,
            Outdated => Self::Outdated,
            Added => Self::Added,
            Modified => Self::Modified,
            Orphaned => Self::Orphaned,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    pub root_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DirTrackingStatus>,
}

pub type RequestBody = Params;
pub type ResponseBody = Outcome;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody { root_url, status } = request_body;
    uc::untrack(
        &pooled_connection,
        collection_uid,
        root_url,
        status.map(Into::into),
    )
    .map(Into::into)
    .map_err(Into::into)
}
