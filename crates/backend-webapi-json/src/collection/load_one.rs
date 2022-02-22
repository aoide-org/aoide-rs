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
use aoide_core_api_json::collection::export_entity_with_summary;
use aoide_usecases_sqlite::collection::load::{self as uc};

use super::*;

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<bool>,
}

pub type ResponseBody = EntityWithSummary;

pub fn handle_request(
    connection: &SqliteConnection,
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
        .transaction::<_, Error, _>(|| {
            uc::load_one(connection, uid, load_scope).map_err(Into::into)
        })
        .map(export_entity_with_summary)
}