// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::collection::LoadScope;
use aoide_core_api_json::collection::EntityWithSummary;

use aoide_usecases_sqlite::collection::load::{self as uc};

use super::*;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
    connection: &mut SqliteConnection,
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
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection| {
    uc::load_all(
            connection,
            kind.as_deref(),
            media_source_root_url,
            load_scope,
            pagination.as_ref(),
            &mut collector,
        )
        //.map_err(Into::into)})
    ?;
    Ok(collector.finish())
}
