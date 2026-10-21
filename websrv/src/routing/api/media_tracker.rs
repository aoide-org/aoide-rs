// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_webapi_json::media::tracker as api;
use aoide_core::CollectionUid;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::media::tracker::{
    Progress, find_untracked_files::ProgressEvent as FindUntrackedProgressEvent,
    import_files::ProgressEvent as ImportProgressEvent,
    scan_directories::ProgressEvent as ScanProgressEvent,
};
use aoide_websrv_axum_sqlite::{
    Json, Path, Result, spawn_blocking_read_task, spawn_blocking_write_task,
};
use axum::{
    Router,
    extract::State,
    routing::{get, post},
};
use tokio::{
    sync::{Mutex, watch},
    task::JoinHandle,
};

use super::{AbortFlag, ApiState};

/// Progress of the currently running media tracker task.
#[derive(Debug, Clone)]
pub(super) struct SharedProgress(Arc<Mutex<Progress>>);

impl Default for SharedProgress {
    fn default() -> Self {
        Self(Arc::new(Mutex::new(Progress::Idle)))
    }
}

impl SharedProgress {
    /// Update the progress from events until the returned sender is dropped.
    ///
    /// The progress is reset to [`Progress::Idle`] when the returned task finishes.
    fn watch<E>(
        self,
        initial: Progress,
        progress_from_event: fn(&E) -> Progress,
    ) -> (watch::Sender<Option<E>>, JoinHandle<()>)
    where
        E: Send + Sync + 'static,
    {
        let Self(progress) = self;
        let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
        let watcher = tokio::spawn(async move {
            *progress.lock().await = initial;
            while progress_event_rx.changed().await.is_ok() {
                let next = progress_event_rx.borrow().as_ref().map(progress_from_event);
                // Borrow has already been released at this point
                if let Some(next) = next {
                    *progress.lock().await = next;
                }
            }
            *progress.lock().await = Progress::Idle;
        });
        (progress_event_tx, watcher)
    }
}

async fn join_watcher(watcher: JoinHandle<()>) {
    if let Err(err) = watcher.await {
        log::error!("Failed to terminate media tracker progress watcher: {err}");
    }
}

pub(super) fn router() -> Router<ApiState> {
    Router::new().route("/progress", get(get_progress))
}

pub(super) fn collected_router() -> Router<ApiState> {
    Router::new()
        .route("/query-status", post(query_status))
        .route("/scan-directories", post(scan_directories))
        .route("/import-files", post(import_files))
        .route("/untrack-directories", post(untrack_directories))
        .route("/find-untracked-files", post(find_untracked_files))
}

async fn get_progress(
    State(SharedProgress(progress)): State<SharedProgress>,
) -> Json<api::Progress> {
    let progress = progress.lock().await.clone();
    Json(progress.into())
}

async fn query_status(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::query_status::RequestBody>,
) -> Result<Json<api::query_status::ResponseBody>> {
    spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        api::query_status::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}

async fn scan_directories(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    State(abort_flag): State<AbortFlag>,
    State(progress): State<SharedProgress>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::scan_directories::RequestBody>,
) -> Result<Json<api::scan_directories::ResponseBody>> {
    let (progress_event_tx, watcher) = progress.watch(
        Progress::Scanning(Default::default()),
        |event: &ScanProgressEvent| Progress::Scanning(event.progress.clone()),
    );
    let response = spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        abort_flag.reset();
        api::scan_directories::handle_request(
            &mut connection,
            &collection_uid,
            request_body,
            &mut |progress_event| {
                progress_event_tx.send_replace(Some(progress_event));
            },
            &abort_flag,
        )
    })
    .await;
    join_watcher(watcher).await;
    response.map(Json)
}

async fn import_files(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    State(abort_flag): State<AbortFlag>,
    State(progress): State<SharedProgress>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::import_files::RequestBody>,
) -> Result<Json<api::import_files::ResponseBody>> {
    let (progress_event_tx, watcher) = progress.watch(
        Progress::Importing(Default::default()),
        |event: &ImportProgressEvent| Progress::Importing(event.summary.clone()),
    );
    let response = spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        abort_flag.reset();
        api::import_files::handle_request(
            &mut connection,
            &collection_uid,
            request_body,
            &mut |progress_event| {
                progress_event_tx.send_replace(Some(progress_event));
            },
            &abort_flag,
        )
    })
    .await;
    join_watcher(watcher).await;
    response.map(Json)
}

async fn untrack_directories(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::untrack_directories::RequestBody>,
) -> Result<Json<api::untrack_directories::ResponseBody>> {
    spawn_blocking_write_task(&gatekeeper, move |mut connection| {
        api::untrack_directories::handle_request(&mut connection, &collection_uid, request_body)
    })
    .await
    .map(Json)
}

async fn find_untracked_files(
    State(gatekeeper): State<Arc<Gatekeeper>>,
    State(abort_flag): State<AbortFlag>,
    State(progress): State<SharedProgress>,
    Path(collection_uid): Path<CollectionUid>,
    Json(request_body): Json<api::find_untracked_files::RequestBody>,
) -> Result<Json<api::find_untracked_files::ResponseBody>> {
    let (progress_event_tx, watcher) = progress.watch(
        Progress::FindingUntracked(Default::default()),
        |event: &FindUntrackedProgressEvent| Progress::FindingUntracked(event.progress.clone()),
    );
    let response = spawn_blocking_read_task(&gatekeeper, move |mut connection| {
        abort_flag.reset();
        api::find_untracked_files::handle_request(
            &mut connection,
            &collection_uid,
            request_body,
            &mut |progress_event| {
                progress_event_tx.send_replace(Some(progress_event));
            },
            &abort_flag,
        )
    })
    .await;
    join_watcher(watcher).await;
    response.map(Json)
}
