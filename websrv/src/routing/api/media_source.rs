// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::media::source as api;
use aoide_core::CollectionUid;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_websrv_axum_sqlite::{Json, Path, Result, spawn_blocking_write_task};
use axum::extract::State;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ApiState;
use crate::openapi as doc;

pub(super) fn collected_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(relocate))
        .routes(routes!(purge_orphaned))
        .routes(routes!(purge_untracked))
}

/// Relocate collected media sources by URI prefix
///
/// Relocate media sources with a common URI prefix by replacing
/// this prefix in their URI with a new prefix.
///
/// The URI comparison is case-sensitive.
#[utoipa::path(
    post,
    path = "/relocate",
    operation_id = "relocateMediaSources",
    tag = doc::COLLECTED_MEDIA_SOURCES_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::RelocateMediaSourcesRequestBody,
    responses(
        (
            status = 200,
            description = "Batch operation succeeded.",
            body = doc::RelocateMediaSourcesResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn relocate(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::relocate::RequestBody>,
) -> Result<Json<api::relocate::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::relocate::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}

/// Purge media sources without a corresponding track
///
/// Purge all media sources that are not associated with a track.
#[utoipa::path(
    post,
    path = "/purge-orphaned",
    operation_id = "purgeOrphanedMediaSources",
    tag = doc::COLLECTED_MEDIA_SOURCES_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::PurgeOrphanedMediaSourcesParams,
    responses(
        (
            status = 200,
            description = "Operation succeeded.",
            body = doc::PurgeOrphanedMediaSourcesOutcome,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn purge_orphaned(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::purge_orphaned::RequestBody>,
) -> Result<Json<api::purge_orphaned::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::purge_orphaned::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}

/// Purge untracked media sources and the corresponding tracks
///
/// Purges all media sources that are not tracked by the media tracker. This
/// will also purge the corresponding tracks and all their transitive relations,
/// i.e. playlist entries.
///
/// Purging all orphaned directories from the media tracker before invoking
/// this operation might generate additional candidates.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/purge-untracked",
    operation_id = "purgeUntrackedMediaSources",
    tag = doc::COLLECTED_MEDIA_SOURCES_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::PurgeUntrackedMediaSourcesParams,
    responses(
        (
            status = 200,
            description = "Batch operation succeeded.",
            body = doc::PurgeUntrackedMediaSourcesOutcome,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn purge_untracked(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::purge_untracked::RequestBody>,
) -> Result<Json<api::purge_untracked::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::purge_untracked::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}
