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
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, patch},
};

use super::ApiState;

pub(super) fn router() -> Router<ApiState> {
    Router::new()
        .route("/", get(load_all).post(create))
        .route("/{playlist_uid}", get(load_one).put(update).delete(purge))
        .route("/{playlist_uid}/entries", patch(patch_entries))
}

pub(super) fn collected_router() -> Router<ApiState> {
    Router::new().route("/", get(load_all_collected).post(create_collected))
}

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
