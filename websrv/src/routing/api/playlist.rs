// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{borrow::Cow, sync::Arc};

use aoide_backend_webapi_json::{EntityRevQueryParams, playlist as api};
use aoide_core::{CollectionUid, PlaylistUid};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::playlist::CollectionFilter;
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
        .routes(routes!(load_one, update, purge))
        .routes(routes!(patch_entries))
}

pub(super) fn collected_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new().routes(routes!(load_all_collected, create_collected))
}

/// Create a new, global playlist
///
/// Create a new global playlist that does not belong to a particular collection.
#[utoipa::path(
    post,
    path = "",
    operation_id = "createPlaylist",
    tag = doc::PLAYLISTS_TAG,
    request_body(
        content = doc::Playlist,
        description = "Properties and entries of the new playlist.",
    ),
    responses(
        (status = 201, description = "The newly created playlist entity.", body = doc::PlaylistEntity),
        (status = 400, response = doc::BadRequest),
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
        api::create::handle_request(&mut connection, None, request_body)
    })
    .await
    .map(|response_body| (StatusCode::CREATED, Json(response_body)))
}

/// Load a single playlist
///
/// Load a single playlist entity with all entries by UID.
#[utoipa::path(
    get,
    path = "/{playlistUid}",
    operation_id = "loadPlaylist",
    tag = doc::PLAYLISTS_TAG,
    params(doc::PlaylistUidPath),
    responses(
        (
            status = 200,
            description = "The loaded playlist entity including all entries.",
            body = doc::PlaylistWithEntriesEntity,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_one(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(playlist_uid): Path<PlaylistUid>,
) -> Result<Json<api::load_one::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_one::handle_request(&mut connection, &playlist_uid)
    })
    .await
    .map(Json)
}

/// List global playlists that do not belong to a particular collection
///
/// Load global playlists that have recently been modified (created/updated).
///
/// For each playlist a summary of all entries is provided.
#[utoipa::path(
    get,
    path = "",
    operation_id = "loadPlaylists",
    tag = doc::PLAYLISTS_TAG,
    params(doc::LoadPlaylistsQuery),
    responses(
        (
            status = 200,
            description = "An array of playlist entities with a summary of their entries.",
            body = doc::LoadPlaylistsResponseBody,
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
        api::load_all::handle_request(&mut connection, None, query_params)
    })
    .await
    .map(Json)
}

/// Update a playlist
///
/// Update a playlist entity after validating that the provided `rev`
/// matches that of the current entity (optimistic locking).
///
/// The entries of the playlist are not affected.
#[utoipa::path(
    put,
    path = "/{playlistUid}",
    operation_id = "updatePlaylist",
    tag = doc::PLAYLISTS_TAG,
    params(doc::PlaylistUidPath, doc::EntityRevQuery),
    request_body(content = doc::Playlist, description = "The updated playlist."),
    responses(
        (status = 200, description = "The updated playlist entity.", body = doc::PlaylistEntity),
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
    Path(playlist_uid): Path<PlaylistUid>,
    Query(query_params): Query<EntityRevQueryParams>,
    Json(request_body): Json<api::update::RequestBody>,
) -> Result<Json<api::update::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::update::handle_request(&mut connection, playlist_uid, query_params, request_body)
    })
    .await
    .map(Json)
}

/// Delete a playlist
///
/// Delete a playlist including all its entries.
#[utoipa::path(
    delete,
    path = "/{playlistUid}",
    operation_id = "deletePlaylist",
    tag = doc::PLAYLISTS_TAG,
    params(doc::PlaylistUidPath),
    responses(
        (status = 204, description = "The playlist has been deleted."),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn purge(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(playlist_uid): Path<PlaylistUid>,
) -> Result<StatusCode> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::purge::handle_request(&mut connection, &playlist_uid)
    })
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

/// Patch playlist entries
///
/// Modify the playlist's entries by applying a sequence of operations.
#[utoipa::path(
    patch,
    path = "/{playlistUid}/entries",
    operation_id = "patchPlaylistEntries",
    tag = doc::PLAYLISTS_TAG,
    params(doc::PlaylistUidPath, doc::EntityRevQuery),
    request_body = doc::PatchPlaylistEntriesRequestBody,
    responses(
        (
            status = 200,
            description = "The updated playlist entity.",
            body = doc::PlaylistWithEntriesSummaryEntity,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 409, response = doc::Conflict),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn patch_entries(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(playlist_uid): Path<PlaylistUid>,
    Query(query_params): Query<EntityRevQueryParams>,
    Json(request_body): Json<api::entries::patch::RequestBody>,
) -> Result<Json<api::entries::patch::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::entries::patch::handle_request(
            &mut connection,
            playlist_uid,
            query_params,
            request_body,
        )
    })
    .await
    .map(Json)
}

/// Create a new, collected playlist
///
/// Create a new playlist in a collection.
#[utoipa::path(
    post,
    path = "",
    operation_id = "createCollectedPlaylist",
    tag = doc::COLLECTED_PLAYLISTS_TAG,
    params(doc::CollectionUidPath),
    request_body(
        content = doc::Playlist,
        description = "Properties and entries of the new playlist.",
    ),
    responses(
        (status = 201, description = "The newly created playlist entity.", body = doc::PlaylistEntity),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 409, response = doc::Conflict),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn create_collected(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::create::RequestBody>,
) -> Result<(StatusCode, Json<api::create::ResponseBody>)> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::create::handle_request(&mut connection, Some(&collection_uid), request_body)
    })
    .await
    .map(|response_body| (StatusCode::CREATED, Json(response_body)))
}

/// List collected playlists
///
/// Load collected playlists that have recently been modified (created/updated).
#[utoipa::path(
    get,
    path = "",
    operation_id = "loadCollectedPlaylists",
    tag = doc::COLLECTED_PLAYLISTS_TAG,
    params(doc::CollectionUidPath, doc::LoadPlaylistsQuery),
    responses(
        (
            status = 200,
            description = "An array of playlist entities with a summary of their entries.",
            body = doc::LoadPlaylistsResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_all_collected(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Query(query_params): Query<api::load_all::QueryParams>,
) -> Result<Json<api::load_all::ResponseBody>> {
    let collection_filter = CollectionFilter {
        uid: Some(Cow::Owned(collection_uid)),
    };
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_all::handle_request(&mut connection, Some(collection_filter), query_params)
    })
    .await
    .map(Json)
}
