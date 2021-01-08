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
    pub use crate::usecases::tracks::replace::*;

    pub use aoide_repo::track::ReplaceMode;
}

pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
};

///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
enum ReplaceMode {
    CreateOnly,
    UpdateOnly,
    UpdateOrCreate,
}

impl From<ReplaceMode> for uc::ReplaceMode {
    fn from(from: ReplaceMode) -> Self {
        use ReplaceMode::*;
        match from {
            CreateOnly => Self::CreateOnly,
            UpdateOnly => Self::UpdateOnly,
            UpdateOrCreate => Self::UpdateOrCreate,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<ReplaceMode>,
}

pub type RequestBody = Vec<Track>;

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResponseBody {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<Entity>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

impl From<uc::Outcome> for ResponseBody {
    fn from(from: uc::Outcome) -> Self {
        let uc::Outcome {
            created,
            updated,
            unchanged,
            not_created,
            not_updated,
        } = from;
        Self {
            created: created.into_iter().map(Into::into).collect(),
            updated: updated.into_iter().map(Into::into).collect(),
            unchanged: unchanged.into_iter().map(Into::into).collect(),
            not_created: not_created.into_iter().map(Into::into).collect(),
            not_updated: not_updated.into_iter().map(Into::into).collect(),
        }
    }
}

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let QueryParams { mode } = query_params;
    let mode = mode.unwrap_or(ReplaceMode::UpdateOrCreate);
    Ok(uc::replace_by_media_source_uri(
        &pooled_connection,
        collection_uid,
        mode.into(),
        request_body.into_iter().map(Into::into),
    )
    .map(Into::into)?)
}
