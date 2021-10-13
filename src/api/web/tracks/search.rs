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

use url::Url;

use aoide_core::util::url::BaseUrl;
use aoide_core_serde::{track::Entity, usecases::track::search::SearchParams};

use super::*;

pub type RequestBody = SearchParams;

pub type ResponseBody = Vec<Entity>;

mod uc {
    pub use crate::usecases::tracks::search::search;
    pub use aoide_usecases::track::search::Params;
}

mod _core {
    pub use aoide_core::entity::EntityUid;
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_url_from_path: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_root_url: Option<Url>,

    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
    // TODO: Replace limit/offset with pagination after serde issue
    // has been fixed: https://github.com/serde-rs/serde/issues/1183
    //#[serde(flatten)]
    //pub pagination: PaginationQueryParams,
}

const DEFAULT_PAGINATION: Pagination = Pagination {
    limit: 100,
    offset: None,
};

#[tracing::instrument(
    name = "Searching tracks",
    skip(
        pooled_connection,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let QueryParams {
        resolve_url_from_path,
        override_root_url,
        limit,
        offset,
    } = query_params;
    let override_root_url = override_root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
    let pagination = PaginationQueryParams { limit, offset };
    let pagination = Option::from(pagination).unwrap_or(DEFAULT_PAGINATION);
    // Passing a base URL override implies resolving paths
    let resolve_url_from_path = override_root_url.is_some()
        || resolve_url_from_path.unwrap_or(uc::Params::default().resolve_url_from_path);
    let params = uc::Params {
        resolve_url_from_path,
        override_root_url,
    };
    let RequestBody { filter, ordering } = request_body;
    let mut collector = EntityCollector::default();
    uc::search(
        pooled_connection,
        collection_uid,
        &pagination,
        filter.map(Into::into),
        ordering.into_iter().map(Into::into).collect(),
        params,
        &mut collector,
    )?;
    Ok(collector.into())
}
