// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::collection::LoadScope;
use aoide_core_api_json::collection::export_entity_with_summary;
use aoide_usecases_sqlite::collection as uc;

use super::*;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<bool>,
}

pub type ResponseBody = EntityWithSummary;

#[allow(clippy::needless_pass_by_value)] // consume arguments
pub fn handle_request(
    connection: &mut DbConnection,
    uid: &EntityUid,
    query_params: QueryParams,
) -> Result<EntityWithSummary> {
    let QueryParams { summary } = query_params;
    let load_scope = if summary.unwrap_or(false) {
        LoadScope::EntityWithSummary
    } else {
        LoadScope::Entity
    };
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::load_one(connection, uid, load_scope).map_err(Into::into)
        })
        .map(export_entity_with_summary)
}
