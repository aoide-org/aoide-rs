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

use aoide_core::util::url::BaseUrl;

use aoide_core_api::media::source::ResolveUrlFromContentPath;
use aoide_core_json::track::Entity;

use aoide_core_api_json::track::search::{QueryParams, SearchParams};

use super::*;

mod uc {
    pub use aoide_core_api::track::search::Params;
    pub use aoide_usecases_sqlite::track::search::search;
}

mod _inner {
    pub use aoide_core::entity::EntityUid;
    pub use aoide_core_api::Pagination;
}

pub type RequestBody = SearchParams;

pub type ResponseBody = Vec<Entity>;

#[tracing::instrument(
    name = "Searching tracks",
    skip(
        connection,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    connection: &SqliteConnection,
    collection_uid: &_inner::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    // TODO: Share common code of search/find_unsynchronized use cases
    // vvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvvv
    let QueryParams {
        resolve_url_from_content_path,
        override_root_url,
        limit,
        offset,
    } = query_params;
    let override_root_url = override_root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
    let pagination = Pagination { limit, offset };
    let pagination = if pagination.is_paginated() {
        pagination
    } else {
        DEFAULT_PAGINATION
    };
    // Passing a base URL override implies resolving paths
    let resolve_url_from_content_path =
        if resolve_url_from_content_path.unwrap_or(false) || override_root_url.is_some() {
            let resolve_url_from_content_path = if let Some(root_url) = override_root_url {
                ResolveUrlFromContentPath::OverrideRootUrl { root_url }
            } else {
                ResolveUrlFromContentPath::CanonicalRootUrl
            };
            Some(resolve_url_from_content_path)
        } else {
            None
        };
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    let RequestBody { filter, ordering } = request_body;
    let params = uc::Params {
        resolve_url_from_content_path,
        filter: filter.map(Into::into),
        ordering: ordering.into_iter().map(Into::into).collect(),
    };
    let mut collector = EntityCollector::default();
    uc::search(
        connection,
        collection_uid,
        params,
        &pagination,
        &mut collector,
    )?;
    Ok(collector.into())
}
