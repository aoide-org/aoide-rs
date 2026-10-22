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
use axum::extract::State;
use tokio::{
    sync::{Mutex, watch},
    task::JoinHandle,
};
use utoipa_axum::{router::OpenApiRouter, routes};

use super::{AbortFlag, ApiState};
use crate::openapi as doc;

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

pub(super) fn router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new().routes(routes!(get_progress))
}

pub(super) fn collected_router() -> OpenApiRouter<ApiState> {
    OpenApiRouter::new()
        .routes(routes!(query_status))
        .routes(routes!(scan_directories))
        .routes(routes!(import_files))
        .routes(routes!(untrack_directories))
        .routes(routes!(find_untracked_files))
}

/// Query the current progress
///
/// Show the currently running batch operation and report the progress so far.
///
/// If no batch operation is running then the string "idle" is returned.
#[utoipa::path(
    get,
    path = "/progress",
    operation_id = "getMediaTrackerProgress",
    tag = doc::MEDIA_TRACKER_TAG,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::MediaTrackerProgress),
        (status = 500, response = doc::InternalServerError),
    ),
)]
async fn get_progress(
    State(SharedProgress(progress)): State<SharedProgress>,
) -> Json<api::Progress> {
    let progress = progress.lock().await.clone();
    Json(progress.into())
}

/// Query the aggregated status of tracked directories
///
/// Fetches the aggregated status of tracked directories from the database.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/query-status",
    operation_id = "queryMediaTrackerStatus",
    tag = doc::COLLECTED_MEDIA_TRACKER_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::MediaTrackerQueryStatusParams,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::MediaTrackerStatus),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
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

/// Scan a directory tree for changes
///
/// Recursively scan a root directory for changes, i.e. discover
/// added/modified/removed files and directories.
///
/// For each directory a cryptographic hash is calculated and stored in
/// the database for further processing, e.g. import of track metadata.
/// Results are stored on behalf of the collection.
///
/// Previously scanned directories that are not discovered again during
/// a subsequent scan are marked and counted as *orphaned*.
///
/// This operation is blocking and locks the database while running.
/// It can be aborted with `/api/storage/abort-current-task`.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/scan-directories",
    operation_id = "scanDirectories",
    tag = doc::COLLECTED_MEDIA_TRACKER_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::MediaTrackerFsTraversalParams,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::MediaTrackerScanDirectoriesOutcome),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
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

/// Import file contents from changed directories
///
/// Recursively import media sources from files located in directories
/// that have been marked as changed.
///
/// This operation is blocking and locks the database while running.
/// It can be aborted with `/api/storage/abort-current-task`.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/import-files",
    operation_id = "importFiles",
    tag = doc::COLLECTED_MEDIA_TRACKER_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::MediaTrackerImportFilesParams,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::MediaTrackerImportFilesOutcome),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
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

/// Discard tracked directories and their contents
///
/// Recursively purge the tracking status of a directory subtree.
///
/// The media sources (and tracks) that are located in the subtree
/// are unaffected. As a result all these media sources and
/// corresponding tracks become *untracked*.
///
/// Untracked directories can be re-added by a rescanning.
/// This would also allow to recover from an inconsistent state
/// i.e. if changes are expected but not detected by rescanning.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/untrack-directories",
    operation_id = "untrackDirectories",
    tag = doc::COLLECTED_MEDIA_TRACKER_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::MediaTrackerUntrackDirectoriesParams,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::MediaTrackerUntrackDirectoriesOutcome),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
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

/// Find untracked entries in the directory tree
///
/// Recursively scan a root directory for untracked files.
///
/// This operation is blocking while running.
/// It can be aborted with `/api/storage/abort-current-task`.
///
/// Requires a collection of local files, i.e. with path kind 3
/// (virtual file path) and a `file://` root URL. Otherwise the request
/// fails with status 500.
#[utoipa::path(
    post,
    path = "/find-untracked-files",
    operation_id = "findUntrackedFiles",
    tag = doc::COLLECTED_MEDIA_TRACKER_TAG,
    params(doc::CollectionUidPath),
    request_body = doc::MediaTrackerFsTraversalParams,
    responses(
        (status = 200, description = "Operation succeeded.", body = doc::MediaTrackerFindUntrackedFilesOutcome),
        (status = 400, response = doc::BadRequest),
        (status = 404, response = doc::NotFound),
        (status = 415, response = doc::UnsupportedMediaType),
        (status = 422, response = doc::UnprocessableEntity),
        (status = 500, response = doc::InternalServerError),
    ),
)]
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
