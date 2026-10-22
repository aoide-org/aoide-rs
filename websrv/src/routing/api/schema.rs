// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON schemas of requests and responses.

use aoide_backend_webapi_json::collection as api;
use axum::Json;
use schemars::schema_for;
use serde_json::{Value, json};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ApiState;
use crate::openapi as doc;

pub(super) fn router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(collection_create))
        .routes(routes!(collection_update))
        .routes(routes!(collection_load_all))
        .routes(routes!(collection_load_all_kinds))
        .routes(routes!(collection_load_one))
}

/// JSON schemas for creating a collection
///
/// The JSON schemas of the request and the response of `POST /api/c`.
///
/// Only available if the server has been built with the feature `json-schema`.
#[utoipa::path(
    get,
    path = "/post/c",
    operation_id = "getCreateCollectionSchemas",
    tag = doc::SCHEMAS_TAG,
    responses(
        (
            status = 200,
            description = "An object with the JSON schemas of the operation, keyed by `query`, \
                `request`, and `response`. Only applicable keys are present.",
            body = Object,
        ),
    ),
)]
async fn collection_create() -> Json<Value> {
    Json(json!({
        "request": schema_for!(api::create::RequestBody),
        "response": schema_for!(api::create::ResponseBody),
    }))
}

/// JSON schemas for updating a collection
///
/// The JSON schemas of the query parameters, the request, and the response
/// of `PUT /api/c/{collectionUid}`. The value of the path parameter is ignored.
///
/// Only available if the server has been built with the feature `json-schema`.
#[utoipa::path(
    get,
    path = "/put/c/{collectionUid}",
    operation_id = "getUpdateCollectionSchemas",
    tag = doc::SCHEMAS_TAG,
    params(doc::CollectionUidPath),
    responses(
        (
            status = 200,
            description = "An object with the JSON schemas of the operation, keyed by `query`, \
                `request`, and `response`. Only applicable keys are present.",
            body = Object,
        ),
    ),
)]
async fn collection_update() -> Json<Value> {
    Json(json!({
        "query": schema_for!(api::update::QueryParams),
        "request": schema_for!(api::update::RequestBody),
        "response": schema_for!(api::update::ResponseBody),
    }))
}

/// JSON schemas for listing collections
///
/// The JSON schemas of the query parameters and the response of `GET /api/c`.
///
/// Only available if the server has been built with the feature `json-schema`.
#[utoipa::path(
    get,
    path = "/get/c",
    operation_id = "getLoadCollectionsSchemas",
    tag = doc::SCHEMAS_TAG,
    responses(
        (
            status = 200,
            description = "An object with the JSON schemas of the operation, keyed by `query`, \
                `request`, and `response`. Only applicable keys are present.",
            body = Object,
        ),
    ),
)]
async fn collection_load_all() -> Json<Value> {
    Json(json!({
        "query": schema_for!(api::load_all::QueryParams),
        "response": schema_for!(api::load_all::ResponseBody),
    }))
}

/// JSON schemas for loading a collection
///
/// The JSON schema of the response of `GET /api/c/{collectionUid}`.
/// The value of the path parameter is ignored.
///
/// Only available if the server has been built with the feature `json-schema`.
#[utoipa::path(
    get,
    path = "/get/c/{collectionUid}",
    operation_id = "getLoadCollectionSchemas",
    tag = doc::SCHEMAS_TAG,
    params(doc::CollectionUidPath),
    responses(
        (
            status = 200,
            description = "An object with the JSON schemas of the operation, keyed by `query`, \
                `request`, and `response`. Only applicable keys are present.",
            body = Object,
        ),
    ),
)]
async fn collection_load_one() -> Json<Value> {
    Json(json!({
        "response": schema_for!(api::load_one::ResponseBody),
    }))
}

/// JSON schemas for listing collection kinds
///
/// The JSON schema of the response of `GET /api/c/kinds`.
///
/// Only available if the server has been built with the feature `json-schema`.
#[utoipa::path(
    get,
    path = "/get/c/kinds",
    operation_id = "getLoadCollectionKindsSchemas",
    tag = doc::SCHEMAS_TAG,
    responses(
        (
            status = 200,
            description = "An object with the JSON schemas of the operation, keyed by `query`, \
                `request`, and `response`. Only applicable keys are present.",
            body = Object,
        ),
    ),
)]
async fn collection_load_all_kinds() -> Json<Value> {
    Json(json!({
        "response": schema_for!(api::load_all_kinds::ResponseBody),
    }))
}
