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

use aoide_usecases_sqlite::SqlitePooledConnection;

use aoide_core::util::url::BaseUrl;

use aoide_core_serde::track::Entity;

use aoide_core_ext_serde::track::search::{QueryParams, SearchParams};

use super::*;

mod uc {
    pub use aoide_core_ext::track::search::Params;
    pub use aoide_usecases_sqlite::track::search::search;
}

mod _inner {
    pub use aoide_core::entity::EntityUid;
    pub use aoide_core_ext::Pagination;
}

pub type RequestBody = SearchParams;

pub type ResponseBody = Vec<Entity>;

const DEFAULT_PAGINATION: _inner::Pagination = _inner::Pagination {
    limit: Some(100),
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
    collection_uid: &_inner::EntityUid,
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
    let pagination = _inner::Pagination { limit, offset };
    let pagination = if pagination.is_paginated() {
        pagination
    } else {
        DEFAULT_PAGINATION
    };
    // Passing a base URL override implies resolving paths
    let resolve_url_from_path = override_root_url.is_some()
        || resolve_url_from_path.unwrap_or(uc::Params::default().resolve_url_from_path);
    let RequestBody { filter, ordering } = request_body;
    let params = uc::Params {
        resolve_url_from_path,
        override_root_url,
        filter: filter.map(Into::into),
        ordering: ordering.into_iter().map(Into::into).collect(),
    };
    let mut collector = EntityCollector::default();
    uc::search(
        pooled_connection,
        collection_uid,
        params,
        &pagination,
        &mut collector,
    )?;
    Ok(collector.into())
}
