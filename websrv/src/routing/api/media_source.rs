// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::media::source as api;
use aoide_core::CollectionUid;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_websrv_axum_sqlite::{Json, Path, Result, spawn_blocking_write_task};
use axum::{Router, extract::State, routing::post};

use super::ApiState;

// TODO: Add OpenAPI docs for all collected media source requests
pub(super) fn collected_router() -> Router<ApiState> {
    Router::new()
        .route("/relocate", post(relocate))
        .route("/purge-orphaned", post(purge_orphaned))
        .route("/purge-untracked", post(purge_untracked))
}

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
