// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
        vfs_content_path_root_url,
        limit,
        offset,
    } = query_params;
    let pagination = Pagination { limit, offset };
    let pagination = if pagination.is_paginated() {
        pagination
    } else {
        DEFAULT_PAGINATION
    };
    let RequestBody { filter, ordering } = request_body;
    let params = uc::Params {
        vfs_content_path_root_url,
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
