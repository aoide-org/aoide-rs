// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_repo_sqlite::DEFAULT_VACUUM_MODE;
use aoide_storage_sqlite::{
    cleanse_database,
    connection::pool::gatekeeper::{Gatekeeper, PendingTasks},
};
use aoide_usecases_sqlite as uc;
use aoide_websrv_axum_sqlite::{Json, Query, Result, spawn_blocking_write_task};
use axum::{extract::State, http::StatusCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{AbortFlag, ApiState};
use crate::openapi as doc;

pub(super) fn router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(get_pending_tasks))
        .routes(routes!(abort_current_task))
        .routes(routes!(migrate_schema))
        .routes(routes!(cleanse))
}

/// The number of pending storage tasks.
#[derive(Serialize, JsonSchema)]
#[schemars(rename = "StoragePendingTasks")]
pub(crate) struct PendingTasksResponseBody {
    /// The number of pending read tasks.
    read: usize,

    /// The number of pending write tasks.
    write: usize,
}

/// Get the number of pending tasks
///
/// Returns the number of pending read and write tasks.
#[utoipa::path(
    get,
    path = "/pending-tasks",
    operation_id = "getPendingStorageTasks",
    tag = doc::STORAGE_TAG,
    responses(
        (status = 200, description = "The number of pending storage tasks.", body = doc::StoragePendingTasks),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn get_pending_tasks(
    State(gatekeeper): State<Arc<Gatekeeper>>,
) -> Json<PendingTasksResponseBody> {
    let PendingTasks { read, write } = gatekeeper.pending_tasks();
    Json(PendingTasksResponseBody { read, write })
}

/// Abort the current task
///
/// Aborts a running storage operation before completion.
///
/// Read tasks may block the database for writers. Write tasks are
/// exclusive and may block both pending readers and writers. In
/// some cases it might be desirable to abort a running task.
///
/// Not all operations support aborting. Batch operations will check
/// periodically if an abort has been requested.
#[utoipa::path(
    post,
    path = "/abort-current-task",
    operation_id = "abortCurrentStorageTask",
    tag = doc::STORAGE_TAG,
    responses(
        (status = 202, description = "The operation has been accepted."),
    ),
)]
async fn abort_current_task(State(abort_flag): State<AbortFlag>) -> StatusCode {
    abort_flag.abort();
    StatusCode::ACCEPTED
}

/// Migrate the database schema
///
/// Migrate the database schema to the most recent version.
#[utoipa::path(
    post,
    path = "/migrate-schema",
    operation_id = "migrateStorageSchema",
    tag = doc::STORAGE_TAG,
    responses(
        (status = 204, description = "The database schema has been migrated."),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn migrate_schema(State(gatekeeper): State<Arc<Gatekeeper>>) -> Result<StatusCode> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        uc::database::migrate_schema(&mut connection)
    })
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

#[derive(Deserialize, JsonSchema)]
pub(crate) struct CleanseQueryParams {
    /// Rebuild the database file from scratch.
    vacuum: bool,
}

/// Cleanse the database
///
/// First check and repair inconsistencies, then analyze and optimize
/// statistics for the query planner.
///
/// Finally, the database could be compacted by rebuilding the database
/// file from scratch.
#[utoipa::path(
    post,
    path = "/cleanse",
    operation_id = "cleanseStorage",
    tag = doc::STORAGE_TAG,
    params(doc::CleanseStorageQuery),
    responses(
        (status = 204, description = "The database has been cleansed."),
        (status = 400, response = doc::BadRequest),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn cleanse(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Query(CleanseQueryParams { vacuum }): Query<CleanseQueryParams>,
) -> Result<StatusCode> {
    let vacuum_mode = vacuum.then_some(DEFAULT_VACUUM_MODE);
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        cleanse_database(&mut connection, vacuum_mode)
    })
    .await
    .map(|()| StatusCode::NO_CONTENT)
}
