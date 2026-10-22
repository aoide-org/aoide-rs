// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::collection as api;
use aoide_core::CollectionUid;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_websrv_axum_sqlite::{
    Json, Path, Query, Result, spawn_blocking_read_task, spawn_blocking_write_task,
};
use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ApiState;
use crate::openapi as doc;

pub(super) fn router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(load_all, create))
        .routes(routes!(load_all_kinds))
        .routes(routes!(load_one, update, purge))
}

/// Create a new collection
///
/// Create a new collection with a unique title.
#[utoipa::path(
    post,
    path = "",
    operation_id = "createCollection",
    tag = doc::COLLECTIONS_TAG,
    request_body(content = doc::Collection, description = "The new collection."),
    responses(
        (status = 201, description = "The newly created collection entity.", body = doc::CollectionEntity),
        (status = 400, response = doc::BadRequest),
        (status = 409, response = doc::Conflict),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn create(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Json(request_body): Json<api::create::RequestBody>,
) -> Result<(StatusCode, Json<api::create::ResponseBody>)> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::create::handle_request(&mut connection, request_body)
    })
    .await
    .map(|response_body| (StatusCode::CREATED, Json(response_body)))
}

/// Update a collection
///
/// Update a collection entity after validating that the provided `rev`
/// matches that of the current entity (optimistic locking).
#[utoipa::path(
    put,
    path = "/{collectionUid}",
    operation_id = "updateCollection",
    tag = doc::COLLECTIONS_TAG,
    params(doc::CollectionUidPath, doc::UpdateCollectionQuery),
    request_body(content = doc::Collection, description = "The updated collection."),
    responses(
        (status = 200, description = "The updated collection entity.", body = doc::CollectionEntity),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 409, response = doc::Conflict),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn update(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Query(query_params): Query<api::update::QueryParams>,
    Json(request_body): Json<api::update::RequestBody>,
) -> Result<Json<api::update::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::update::handle_request(&mut connection, collection_uid, query_params, request_body)
    })
    .await
    .map(Json)
}

/// Delete a collection
///
/// Delete a collection including all its media sources, tracks, and playlists.
#[utoipa::path(
    delete,
    path = "/{collectionUid}",
    operation_id = "deleteCollection",
    tag = doc::COLLECTIONS_TAG,
    params(doc::CollectionUidPath),
    responses(
        (status = 204, description = "The collection has been deleted."),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn purge(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
) -> Result<StatusCode> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::purge::handle_request(&mut connection, &collection_uid)
    })
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

/// List collections
///
/// Load collections that have recently been modified (created/updated).
#[utoipa::path(
    get,
    path = "",
    operation_id = "loadCollections",
    tag = doc::COLLECTIONS_TAG,
    params(doc::LoadCollectionsQuery),
    responses(
        (
            status = 200,
            description = "An array of collection entities, optionally with a summary.",
            body = doc::LoadCollectionsResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_all(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Query(query_params): Query<api::load_all::QueryParams>,
) -> Result<Json<api::load_all::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_all::handle_request(&mut connection, query_params)
    })
    .await
    .map(Json)
}

/// Load a single collection
///
/// Load a single collection entity by UID, including its summary.
#[utoipa::path(
    get,
    path = "/{collectionUid}",
    operation_id = "loadCollection",
    tag = doc::COLLECTIONS_TAG,
    params(doc::CollectionUidPath),
    responses(
        (
            status = 200,
            description = "The loaded collection entity with a summary.",
            body = doc::CollectionWithSummaryEntity,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_one(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
) -> Result<Json<api::load_one::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_one::handle_request(&mut connection, &collection_uid)
    })
    .await
    .map(Json)
}

/// List collection kinds
///
/// Load all (distinct) kinds of collections.
#[utoipa::path(
    get,
    path = "/kinds",
    operation_id = "loadCollectionKinds",
    tag = doc::COLLECTIONS_TAG,
    responses(
        (
            status = 200,
            description = "An array of distinct collection kinds in no particular order.",
            body = doc::LoadCollectionKindsResponseBody,
        ),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_all_kinds(
    State(gatekeeper): State<Arc<Gatekeeper>>,
) -> Result<Json<api::load_all_kinds::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_all_kinds::handle_request(&mut connection)
    })
    .await
    .map(Json)
}
