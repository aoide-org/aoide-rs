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

use aoide_core_ext_serde::collection::EntityWithSummary;

use crate::usecases::collection::load as uc;

use super::*;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<bool>,

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
    pooled_connection: SqlitePooledConnection,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams {
        kind,
        summary,
        limit,
        offset,
    } = query_params;
    let with_summary = summary.unwrap_or(false);
    let pagination = PaginationQueryParams { limit, offset };
    let pagination: Option<_> = pagination.into();
    let mut collector = EntityCollector::default();
    uc::load_all(
        &pooled_connection,
        kind.as_deref(),
        with_summary,
        pagination.as_ref(),
        &mut collector,
    )?;
    Ok(collector.into())
}
