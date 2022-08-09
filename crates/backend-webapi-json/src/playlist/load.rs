// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api_json::playlist::EntityWithEntriesSummary;
use aoide_usecases_sqlite::playlist::load as uc;

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
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams {
        kind,
        limit,
        offset,
    } = query_params;
    let pagination = Pagination { limit, offset };
    let pagination: Option<_> = pagination.into();
    let mut collector = EntityWithEntriesSummaryCollector::default();
    connection.transaction::<_, Error, _>(|| {
        uc::load_entities_with_entries_summary(
            connection,
            collection_uid,
            kind.as_deref(),
            pagination.as_ref(),
            &mut collector,
        )
        .map_err(Into::into)
    })?;
    Ok(collector.finish())
}
