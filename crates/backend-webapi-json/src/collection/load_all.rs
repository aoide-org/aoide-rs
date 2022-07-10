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

use aoide_core_api::collection::LoadScope;
use aoide_core_api_json::collection::EntityWithSummary;

use aoide_usecases_sqlite::collection::load::{self as uc};

use super::*;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
    // TODO: Replace limit/offset with pagination after serde issue
    // has been fixed: https://github.com/serde-rs/serde/issues/1183
    //#[serde(flatten)]
    //pub pagination: PaginationQueryParams,
}

pub type ResponseBody = Vec<EntityWithSummary>;

pub fn handle_request(
    connection: &SqliteConnection,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams {
        kind,
        summary,
        limit,
        offset,
    } = query_params;
    // TODO: Optionally filter by media source root URL
    let media_source_root_url = None;
    let load_scope = if summary.unwrap_or(false) {
        LoadScope::EntityWithSummary
    } else {
        LoadScope::Entity
    };
    let pagination = Pagination { limit, offset };
    let pagination: Option<_> = pagination.into();
    let mut collector = EntityWithSummaryCollector::default();
    connection.transaction::<_, Error, _>(|| {
        uc::load_all(
            connection,
            kind.as_deref(),
            media_source_root_url,
            load_scope,
            pagination.as_ref(),
            &mut collector,
        )
        .map_err(Into::into)
    })?;
    Ok(collector.finish())
}
