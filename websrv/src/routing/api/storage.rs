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
use axum::{
    Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

use super::{AbortFlag, ApiState};

pub(super) fn router() -> Router<ApiState> {
    Router::new()
        .route("/pending-tasks", get(get_pending_tasks))
        .route("/abort-current-task", post(abort_current_task))
        .route("/migrate-schema", post(migrate_schema))
        .route("/cleanse", post(cleanse))
}

#[derive(Serialize)]
struct PendingTasksResponseBody {
    read: usize,
    write: usize,
}

async fn get_pending_tasks(
    State(gatekeeper): State<Arc<Gatekeeper>>,
) -> Json<PendingTasksResponseBody> {
    let PendingTasks { read, write } = gatekeeper.pending_tasks();
    Json(PendingTasksResponseBody { read, write })
}

async fn abort_current_task(State(abort_flag): State<AbortFlag>) -> StatusCode {
    abort_flag.abort();
    StatusCode::ACCEPTED
}

async fn migrate_schema(State(gatekeeper): State<Arc<Gatekeeper>>) -> Result<StatusCode> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        uc::database::migrate_schema(&mut connection)
    })
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct CleanseQueryParams {
    vacuum: bool,
}

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
