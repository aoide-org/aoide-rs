// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::url::BaseUrl;

use aoide_core_api::media::source::ResolveUrlFromContentPath;

use aoide_core_json::track::Entity;

use aoide_core_api_json::track::search::{QueryParams, SearchParams};

use super::*;

mod uc {
    pub(super) use aoide_core_api::track::search::Params;
    pub(super) use aoide_usecases_sqlite::track::search::search;
}

pub type RequestBody = SearchParams;

pub type ResponseBody = Vec<Entity>;

#[allow(clippy::panic_in_result_fn)] // tracing::instrument
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
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
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
    connection.transaction::<_, Error, _>(|connection| {
        uc::search(
            connection,
            collection_uid,
            params,
            &pagination,
            &mut collector,
        )
        .map_err(Into::into)
    })?;
    Ok(collector.into())
}
