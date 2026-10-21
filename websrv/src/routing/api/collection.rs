// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::collection as api;
use aoide_core::CollectionUid;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_websrv_axum_sqlite::{
    Json, Path, Query, Result, spawn_blocking_read_task, spawn_blocking_write_task,
};
use axum::{Router, extract::State, http::StatusCode, routing::get};

use super::ApiState;

pub(super) fn router() -> Router<ApiState> {
    Router::new()
        .route("/", get(load_all).post(create))
        .route("/kinds", get(load_all_kinds))
        .route("/{collection_uid}", get(load_one).put(update).delete(purge))
}

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

async fn load_all_kinds(
    State(gatekeeper): State<Arc<Gatekeeper>>,
) -> Result<Json<api::load_all_kinds::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::load_all_kinds::handle_request(&mut connection)
    })
    .await
    .map(Json)
}
