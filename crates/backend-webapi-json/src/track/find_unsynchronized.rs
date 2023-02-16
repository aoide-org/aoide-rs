// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api_json::{
    filtering::StringPredicate,
    track::{find_unsynchronized::UnsynchronizedTrackEntity, search::QueryParams},
};

use super::*;

mod uc {
    pub(super) use aoide_core_api::track::find_unsynchronized::Params;
    pub(super) use aoide_usecases_sqlite::track::find_unsynchronized::*;
}

pub type RequestBody = Option<StringPredicate>;

pub type ResponseBody = Vec<UnsynchronizedTrackEntity>;

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
    // ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    let params = uc::Params {
        vfs_content_path_root_url,
        content_path_predicate: request_body.map(Into::into),
    };
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::find_unsynchronized(connection, collection_uid, params, &pagination)
                .map_err(Into::into)
        })
        .map(|v| v.into_iter().map(Into::into).collect())
}
