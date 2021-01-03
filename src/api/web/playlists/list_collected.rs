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

use crate::usecases::playlists::load as uc;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(flatten)]
    pub pagination: PaginationQueryParams,
}

pub type ResponseBody = Vec<EntityWithEntriesSummary>;

pub fn handle_request(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: &EntityUid,
    query_params: QueryParams,
) -> RepoResult<ResponseBody> {
    let QueryParams { kind, pagination } = query_params;
    let pagination: Option<_> = pagination.into();
    let mut collector = EntityWithEntriesSummaryCollector::default();
    uc::load_entities_with_entries_summary(
        pooled_connection,
        collection_uid,
        kind.as_deref(),
        pagination.as_ref(),
        &mut collector,
    )?;
    Ok(collector.into())
}
