// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON schemas of requests and responses.

use aoide_backend_webapi_json::collection as api;
use axum::{Json, Router, routing::get};
use schemars::schema_for;
use serde_json::{Value, json};

use super::ApiState;

pub(super) fn router() -> Router<ApiState> {
    Router::new()
        .route("/post/c", get(collection_create))
        .route("/put/c/{collection_uid}", get(collection_update))
        .route("/get/c", get(collection_load_all))
        .route("/get/c/kinds", get(collection_load_all_kinds))
        .route("/get/c/{collection_uid}", get(collection_load_one))
}

async fn collection_create() -> Json<Value> {
    Json(json!({
        "request": schema_for!(api::create::RequestBody),
        "response": schema_for!(api::create::ResponseBody),
    }))
}

async fn collection_update() -> Json<Value> {
    Json(json!({
        "query": schema_for!(api::update::QueryParams),
        "request": schema_for!(api::update::RequestBody),
        "response": schema_for!(api::update::ResponseBody),
    }))
}

async fn collection_load_all() -> Json<Value> {
    Json(json!({
        "query": schema_for!(api::load_all::QueryParams),
        "response": schema_for!(api::load_all::ResponseBody),
    }))
}

async fn collection_load_one() -> Json<Value> {
    Json(json!({
        "response": schema_for!(api::load_one::ResponseBody),
    }))
}

async fn collection_load_all_kinds() -> Json<Value> {
    Json(json!({
        "response": schema_for!(api::load_all_kinds::ResponseBody),
    }))
}
