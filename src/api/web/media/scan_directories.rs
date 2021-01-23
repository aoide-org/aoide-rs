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
pub struct QueryParams {
    pub root_dir_url: Url,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DirectoryScanOutcome {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

impl From<uc::DirectoryScanOutcome> for DirectoryScanOutcome {
    fn from(from: uc::DirectoryScanOutcome) -> Self {
        let uc::DirectoryScanOutcome {
            current,
            added,
            modified,
            orphaned,
        } = from;
        Self {
            current,
            added,
            modified,
            orphaned,
        }
    }
}

pub type ResponseBody = DirectoryScanOutcome;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams { root_dir_url } = query_params;
    Ok(
        uc::scan_directories_recursively(&pooled_connection, collection_uid, &root_dir_url)
            .map(Into::into)?,
    )
}
