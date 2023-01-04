// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api_json::playlist::EntityWithEntriesSummary;
use aoide_repo::playlist::KindFilter;
use aoide_usecases_sqlite::playlist::load::{self as uc, CollectionFilter};

use super::*;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
    // TODO: Replace limit/offset with pagination after serde issue
    // has been fixed: https://github.com/serde-rs/serde/issues/1183
    //#[serde(flatten)]
    //pub pagination: PaginationQueryParams,
}

pub type ResponseBody = Vec<EntityWithEntriesSummary>;

pub fn handle_request(
    connection: &mut DbConnection,
    collection_filter: Option<CollectionFilter<'_>>,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams {
        kind,
        limit,
        offset,
    } = query_params;
    let kind_filter = kind.map(|kind| KindFilter {
        kind: Some(kind.into()),
    });
    let pagination = Pagination { limit, offset };
    let pagination: Option<_> = pagination.into();
    let mut collector = EntityWithEntriesSummaryCollector::default();
    connection.transaction::<_, Error, _>(|connection| {
        uc::load_all_with_entries_summary(
            connection,
            collection_filter,
            kind_filter,
            pagination.as_ref(),
            &mut collector,
        )
        .map_err(Into::into)
    })?;
    Ok(collector.finish())
}
