// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::track as api;
use aoide_core::{CollectionUid, TrackUid};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_websrv_axum_sqlite::{
    Json, Path, Query, Result, spawn_blocking_read_task, spawn_blocking_write_task,
};
use axum::{extract::State, http::StatusCode};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{AbortFlag, ApiState};
use crate::openapi as doc;

pub(super) fn router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(load_many))
        .routes(routes!(load_one))
        .routes(routes!(export_metadata))
}

pub(super) fn collected_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(resolve))
        .routes(routes!(search))
        .routes(routes!(replace))
        .routes(routes!(import_and_replace))
        .routes(routes!(find_unsynchronized))
        .routes(routes!(export_vfs))
}

/// Load a single track
///
/// Load a single track entity with its associated media source by UID.
#[utoipa::path(
    get,
    path = "/{trackUid}",
    operation_id = "loadTrack",
    tag = doc::TRACKS_TAG,
    params(doc::TrackUidPath),
    responses(
        (status = 200, description = "The loaded track entity.", body = doc::TrackBodyEntity),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_one(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(track_uid): Path<TrackUid>,
) -> Result<Json<api::load_one::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_one::handle_request(&mut connection, &track_uid)
    })
    .await
    .map(Json)
}

/// Load multiple tracks
///
/// Load multiple track entities including their media sources.
///
/// The result may contain fewer tracks than requested if some
/// tracks do not exist. The order of the given UIDs is not preserved
/// in the result set, i.e. the ordering of tracks is undefined!
#[utoipa::path(
    post,
    path = "/load",
    operation_id = "loadTracks",
    tag = doc::TRACKS_TAG,
    request_body(content = doc::LoadTracksRequestBody, description = "The UIDs of the tracks."),
    responses(
        (
            status = 200,
            description = "An unordered array of loaded tracks.",
            body = doc::LoadTracksResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn load_many(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Json(request_body): Json<api::load_many::RequestBody>,
) -> Result<Json<api::load_many::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_many::handle_request(&mut connection, request_body)
    })
    .await
    .map(Json)
}

/// Export metadata of a single track
///
/// Write track metadata into file tags.
#[utoipa::path(
    post,
    path = "/{trackUid}/export-metadata",
    operation_id = "exportTrackMetadata",
    tag = doc::TRACKS_TAG,
    params(doc::TrackUidPath, doc::ExportTrackMetadataQuery),
    responses(
        (status = 204, description = "The metadata has been exported."),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn export_metadata(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(track_uid): Path<TrackUid>,
    Query(query_params): Query<api::export_metadata::QueryParams>,
) -> Result<StatusCode> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::export_metadata::handle_request(&mut connection, &track_uid, query_params)
    })
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

/// Resolve the UIDs of tracks from a list of media source URIs
///
/// Resolve the UID and current revision of tracks from a list of
/// media source URIs.
///
/// The result list is unordered and contains only those tracks
/// that have been found.
#[utoipa::path(
    post,
    path = "/resolve",
    operation_id = "resolveTracks",
    tag = doc::COLLECTED_TRACKS_TAG,
    params(doc::CollectionUidPath),
    request_body(
        content = doc::ResolveTracksRequestBody,
        description = "The (percent-encoded) media source URIs.",
    ),
    responses(
        (
            status = 200,
            description = "An array of track [URI, [UID, Revision]] tuples.",
            body = doc::ResolveTracksResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn resolve(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::resolve::RequestBody>,
) -> Result<Json<api::resolve::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::resolve::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}

/// Search collected tracks and their media sources
///
/// Search collected tracks and their media sources by evaluating
/// filter conditions and sorting the results.
#[utoipa::path(
    post,
    path = "/search",
    operation_id = "searchTracks",
    tag = doc::COLLECTED_TRACKS_TAG,
    params(doc::CollectionUidPath, doc::SearchTracksQuery),
    request_body = doc::TrackSearchParams,
    responses(
        (
            status = 200,
            description = "An array of matching tracks in the requested order.",
            body = doc::SearchTracksResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn search(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Query(query_params): Query<api::search::QueryParams>,
    Json(request_body): Json<api::search::RequestBody>,
) -> Result<Json<api::search::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::search::handle_request(&mut connection, &collection_uid, query_params, request_body)
    })
    .await
    .map(Json)
}

/// Replace collected tracks and media sources by URI
///
/// Create or update multiple, collected tracks and their media sources by locating them
/// through their (percent-encoded) URI.
#[utoipa::path(
    post,
    path = "/replace",
    operation_id = "replaceTracks",
    tag = doc::COLLECTED_TRACKS_TAG,
    params(doc::CollectionUidPath, doc::ReplaceTracksQuery),
    request_body = doc::ReplaceTracksRequestBody,
    responses(
        (status = 200, description = "Batch operation succeeded.", body = doc::ReplaceTracksSummary),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn replace(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Query(query_params): Query<api::replace::QueryParams>,
    Json(request_body): Json<api::replace::RequestBody>,
) -> Result<Json<api::replace::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::replace::handle_request(&mut connection, &collection_uid, query_params, request_body)
    })
    .await
    .map(Json)
}

/// Import and replace collected tracks from local files
///
/// Import the media sources of the given files and create or update
/// the corresponding tracks within a single database transaction.
///
/// The request body contains the content paths of the files, i.e.
/// virtual file paths relative to the root URL of the collection.
///
/// The operation can be aborted with `/api/storage/abort-current-task`.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/import-and-replace",
    operation_id = "importAndReplaceTracks",
    tag = doc::COLLECTED_TRACKS_TAG,
    params(doc::CollectionUidPath, doc::ImportAndReplaceTracksQuery),
    request_body = doc::ImportAndReplaceTracksRequestBody,
    responses(
        (
            status = 200,
            description = "Batch operation succeeded.",
            body = doc::ImportAndReplaceTracksOutcome,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn import_and_replace(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    State(abort_flag): State<AbortFlag>,
    Path(collection_uid): Path<CollectionUid>,
    Query(query_params): Query<api::import_and_replace::QueryParams>,
    Json(request_body): Json<api::import_and_replace::RequestBody>,
) -> Result<Json<api::import_and_replace::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        abort_flag.reset();
        api::import_and_replace::handle_request(
            &mut connection,
            &collection_uid,
            query_params,
            request_body,
            &abort_flag,
        )
    })
    .await
    .map(Json)
}

/// Find tracks with unsynchronized media sources
///
/// Return information about tracks that are not synchronized
/// with their media sources. This includes all tracks that have
/// either never been imported from sources or that have been
/// modified after the last import. The metadata of the latter
/// is supposed to be exported to keep the sources synchronized.
///
/// The optional request body filters the media source paths.
#[utoipa::path(
    post,
    path = "/find-unsynchronized",
    operation_id = "findUnsynchronizedTracks",
    tag = doc::COLLECTED_TRACKS_TAG,
    params(doc::CollectionUidPath, doc::FindUnsynchronizedTracksQuery),
    request_body = doc::FindUnsynchronizedTracksRequestBody,
    responses(
        (
            status = 200,
            description = "An array of unsynchronized tracks.",
            body = doc::FindUnsynchronizedTracksResponseBody,
        ),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn find_unsynchronized(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Query(query_params): Query<api::find_unsynchronized::QueryParams>,
    Json(request_body): Json<api::find_unsynchronized::RequestBody>,
) -> Result<Json<api::find_unsynchronized::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::find_unsynchronized::handle_request(
            &mut connection,
            &collection_uid,
            query_params,
            request_body,
        )
    })
    .await
    .map(Json)
}

/// Export the files of collected tracks into a directory
///
/// Copy the media files of all collected tracks that match an optional
/// filter into a target directory, preserving their virtual file paths
/// relative to the root of the collection.
///
/// Files that already exist in the target directory with the same
/// content are skipped.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/export-vfs",
    operation_id = "exportTrackFiles",
    tag = doc::COLLECTED_TRACKS_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::ExportTrackFilesRequestBody,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::ExportTrackFilesResponseBody),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn export_vfs(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::vfs::export_files::RequestBody>,
) -> Result<Json<api::vfs::export_files::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::vfs::export_files::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}
