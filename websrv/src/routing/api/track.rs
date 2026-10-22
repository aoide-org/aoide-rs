// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::track as api;
use aoide_core::{CollectionUid, TrackUid};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_websrv_axum_sqlite::{
    Json, Path, Query, Result, spawn_blocking_read_task, spawn_blocking_write_task,
};
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};

use super::{AbortFlag, ApiState};

pub(super) fn router() -> Router<ApiState> {
    Router::new()
        .route("/load", post(load_many))
        .route("/{track_uid}", get(load_one))
        .route("/{track_uid}/export-metadata", post(export_metadata))
}

pub(super) fn collected_router() -> Router<ApiState> {
    Router::new()
        .route("/resolve", post(resolve))
        .route("/search", post(search))
        .route("/replace", post(replace))
        .route("/import-and-replace", post(import_and_replace))
        .route("/find-unsynchronized", post(find_unsynchronized))
        // TODO: Add API docs.
        .route("/export-vfs", post(export_vfs))
}

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
