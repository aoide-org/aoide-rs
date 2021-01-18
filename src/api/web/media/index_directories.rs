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

use std::path::Path;

use super::*;

mod uc {
    pub use crate::usecases::media::*;
}

use aoide_core_serde::media::Digest as SerdeDigest;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    pub root_path: String,
    pub expected_count: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct PathWithDigest(String, SerdeDigest);

impl From<uc::PathWithDigest> for PathWithDigest {
    fn from(from: uc::PathWithDigest) -> Self {
        let uc::PathWithDigest { path, digest } = from;
        Self(
            path.to_string_lossy().to_string(),
            SerdeDigest::encode(&digest),
        )
    }
}

pub type ResponseBody = Vec<PathWithDigest>;

pub fn handle_request(query_params: QueryParams) -> Result<ResponseBody> {
    let QueryParams {
        root_path,
        expected_count,
    } = query_params;
    let root_path = Path::new(&root_path);
    let expected_number_of_directories = expected_count.unwrap_or(16_384).min(65_536) as usize;
    Ok(
        uc::index_directories_recursively(root_path, expected_number_of_directories)
            .map(|v| v.into_iter().map(Into::into).collect())?,
    )
}
