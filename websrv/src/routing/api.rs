// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use aoide_backend_webapi_json as api;
use aoide_core::{CollectionUid, PlaylistUid, TrackUid};
use aoide_repo_sqlite::DEFAULT_VACUUM_MODE;
use aoide_storage_sqlite::{
    cleanse_database,
    connection::pool::gatekeeper::{Gatekeeper as DatabaseConnectionGatekeeper, PendingTasks},
};
use aoide_usecases::{
    media::tracker::{
        Progress as MediaTrackerProgress,
        find_untracked_files::ProgressEvent as FindUntrackedProgressEvent,
        import_files::ProgressEvent as ImportProgressEvent,
        scan_directories::ProgressEvent as ScanProgressEvent,
    },
    playlist::CollectionFilter,
};
use aoide_usecases_sqlite as uc;
use aoide_websrv_axum_sqlite::{self as websrv, JsonBody, PathParam, QueryParams};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
#[cfg(feature = "json-schema")]
use schemars::schema_for;
use tokio::sync::{Mutex, watch};

#[derive(Clone)]
struct ApiState {
    rt: tokio::runtime::Handle,
    shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>,
    abort_flag: Arc<AtomicBool>,
    media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>,
}

// TODO: Move into separate request handler
#[derive(serde::Deserialize)]
struct CleanseDatabaseQueryParams {
    vacuum: bool,
}

pub(crate) fn create_router(
    rt: &tokio::runtime::Handle,
    shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>,
    abort_flag: Arc<AtomicBool>,
) -> Router {
    log::info!("Creating API routes");

    let state = ApiState {
        rt: rt.clone(),
        shared_connection_gatekeeper,
        abort_flag,
        media_tracker_progress: Arc::new(Mutex::new(MediaTrackerProgress::Idle)),
    };

    let router = Router::new()
        // Collections
        .route("/c", get(collections_load_all).post(collections_create))
        .route("/c/kinds", get(collections_load_all_kinds))
        .route(
            "/c/{uid}",
            get(collections_load_one)
                .put(collections_update)
                .delete(collections_delete),
        )
        // Media tracker
        .route("/mt/progress", get(media_tracker_get_progress))
        .route(
            "/c/{uid}/mt/query-status",
            post(media_tracker_post_collection_query_status),
        )
        .route(
            "/c/{uid}/mt/scan-directories",
            post(media_tracker_post_collection_scan),
        )
        .route(
            "/c/{uid}/mt/import-files",
            post(media_tracker_post_collection_import),
        )
        .route(
            "/c/{uid}/mt/untrack-directories",
            post(media_tracker_post_collection_untrack),
        )
        .route(
            "/c/{uid}/mt/find-untracked-files",
            post(media_tracker_post_collection_find_untracked_files),
        )
        // Collected media sources
        // TODO: Add OpenAPI docs for all collected media source requests
        .route(
            "/c/{uid}/ms/relocate",
            post(collected_media_sources_relocate),
        )
        .route(
            "/c/{uid}/ms/purge-orphaned",
            post(collected_media_sources_purge_orphaned),
        )
        .route(
            "/c/{uid}/ms/purge-untracked",
            post(collected_media_sources_purge_untracked),
        )
        // Collected tracks
        .route("/c/{uid}/t/resolve", post(collected_tracks_resolve))
        .route("/c/{uid}/t/search", post(collected_tracks_search))
        .route("/c/{uid}/t/replace", post(collected_tracks_replace))
        .route(
            "/c/{uid}/t/import-and-replace",
            post(collected_tracks_import_and_replace),
        )
        .route(
            "/c/{uid}/t/find-unsynchronized",
            post(collected_tracks_find_unsynchronized),
        )
        // TODO: Add API docs.
        .route("/c/{uid}/t/export-vfs", post(collected_tracks_export_vfs))
        // Collected playlists
        .route(
            "/c/{uid}/p",
            get(collected_playlists_load_all).post(collected_playlists_create),
        )
        // Tracks
        .route("/t/load", post(tracks_load_many))
        .route("/t/{uid}", get(tracks_load_one))
        .route("/t/{uid}/export-metadata", post(tracks_export_metadata))
        // Playlists
        .route("/p", get(playlists_load_all).post(playlists_create))
        .route(
            "/p/{uid}",
            get(playlists_load_one)
                .put(playlists_update)
                .delete(playlists_delete),
        )
        .route(
            "/p/{uid}/entries",
            axum::routing::patch(playlists_entries_patch),
        )
        // Storage
        .route("/storage/pending-tasks", get(storage_get_pending_tasks))
        .route(
            "/storage/abort-current-task",
            post(storage_post_abort_current_task),
        )
        .route("/storage/migrate-schema", post(storage_migrate_schema))
        .route("/storage/cleanse", post(storage_cleanse));

    #[cfg(feature = "json-schema")]
    let router = router
        .route("/schema/post/c", get(collections_create_schema))
        .route("/schema/put/c/{uid}", get(collections_update_schema))
        .route("/schema/get/c", get(collections_load_all_schema))
        .route(
            "/schema/get/c/kinds",
            get(collections_load_all_kinds_schema),
        )
        .route("/schema/get/c/{uid}", get(collections_load_one_schema));

    router.with_state(state)
}

// Collections

async fn collections_create(
    State(state): State<ApiState>,
    JsonBody(request_body): JsonBody<api::collection::create::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::collection::create::handle_request(&mut pooled_connection, request_body)
        },
    )
    .await
    .map(|response_body| (StatusCode::CREATED, Json(response_body)))
}

#[cfg(feature = "json-schema")]
async fn collections_create_schema() -> impl IntoResponse {
    let request_schema = schema_for!(api::collection::create::RequestBody);
    let response_schema = schema_for!(api::collection::create::ResponseBody);
    Json(serde_json::json!({
        "request": request_schema,
        "response": response_schema,
    }))
}

async fn collections_update(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    QueryParams(query_params): QueryParams<api::collection::update::QueryParams>,
    JsonBody(request_body): JsonBody<api::collection::update::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::collection::update::handle_request(
                &mut pooled_connection,
                uid,
                query_params,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

#[cfg(feature = "json-schema")]
async fn collections_update_schema(PathParam(_uid): PathParam<CollectionUid>) -> impl IntoResponse {
    let query_schema = schema_for!(api::collection::update::QueryParams);
    let request_schema = schema_for!(api::collection::update::RequestBody);
    let response_schema = schema_for!(api::collection::update::ResponseBody);
    Json(serde_json::json!({
        "query": query_schema,
        "request": request_schema,
        "response": response_schema,
    }))
}

async fn collections_delete(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::collection::purge::handle_request(&mut pooled_connection, &uid)
        },
    )
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

async fn collections_load_all(
    State(state): State<ApiState>,
    QueryParams(query_params): QueryParams<api::collection::load_all::QueryParams>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::collection::load_all::handle_request(&mut pooled_connection, query_params)
        },
    )
    .await
    .map(Json)
}

#[cfg(feature = "json-schema")]
async fn collections_load_all_schema() -> impl IntoResponse {
    let query_schema = schema_for!(api::collection::load_all::QueryParams);
    let response_schema = schema_for!(api::collection::load_all::ResponseBody);
    Json(serde_json::json!({
        "query": query_schema,
        "response": response_schema,
    }))
}

async fn collections_load_one(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::collection::load_one::handle_request(&mut pooled_connection, &uid)
        },
    )
    .await
    .map(Json)
}

#[cfg(feature = "json-schema")]
async fn collections_load_one_schema(
    PathParam(_uid): PathParam<CollectionUid>,
) -> impl IntoResponse {
    let response_schema = schema_for!(api::collection::load_one::ResponseBody);
    Json(serde_json::json!({
        "response": response_schema,
    }))
}

async fn collections_load_all_kinds(
    State(state): State<ApiState>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::collection::load_all_kinds::handle_request(&mut pooled_connection)
        },
    )
    .await
    .map(Json)
}

#[cfg(feature = "json-schema")]
async fn collections_load_all_kinds_schema() -> impl IntoResponse {
    let response_schema = schema_for!(api::collection::load_all_kinds::ResponseBody);
    Json(serde_json::json!({
        "response": response_schema,
    }))
}

// Media tracker

async fn media_tracker_get_progress(State(state): State<ApiState>) -> impl IntoResponse {
    let progress = state.media_tracker_progress.lock().await.clone();
    Json(api::media::tracker::Progress::from(progress))
}

async fn media_tracker_post_collection_query_status(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::tracker::query_status::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::media::tracker::query_status::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn media_tracker_post_collection_scan(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::tracker::scan_directories::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    let ApiState {
        rt,
        shared_connection_gatekeeper,
        abort_flag,
        media_tracker_progress,
    } = state;
    let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
    let watcher = rt.spawn(async move {
        *media_tracker_progress.lock().await = MediaTrackerProgress::Scanning(Default::default());
        log::debug!("Watching media tracker scanning");
        while progress_event_rx.changed().await.is_ok() {
            let progress = progress_event_rx
                .borrow()
                .as_ref()
                .map(|event: &ScanProgressEvent| event.progress.clone());
            // Borrow has already been released at this point
            if let Some(progress) = progress {
                *media_tracker_progress.lock().await = MediaTrackerProgress::Scanning(progress);
            }
        }
        log::debug!("Unwatching media tracker scanning");
        *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
    });
    let response = websrv::spawn_blocking_write_task(
        &shared_connection_gatekeeper,
        move |mut pooled_connection| {
            abort_flag.store(false, Ordering::Relaxed);
            api::media::tracker::scan_directories::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
                &mut |progress_event: ScanProgressEvent| {
                    if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                        log::error!(
                            "Failed to send media tracker scanning progress event: {:?}",
                            err.0
                        );
                    }
                },
                &abort_flag,
            )
        },
    )
    .await;
    if let Err(err) = watcher.await {
        log::error!("Failed to terminate media tracker scanning progress watcher: {err}");
    }
    response.map(Json)
}

async fn media_tracker_post_collection_import(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::tracker::import_files::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    let ApiState {
        rt,
        shared_connection_gatekeeper,
        abort_flag,
        media_tracker_progress,
    } = state;
    let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
    let watcher = rt.spawn(async move {
        *media_tracker_progress.lock().await = MediaTrackerProgress::Importing(Default::default());
        log::debug!("Watching media tracker importing");
        while progress_event_rx.changed().await.is_ok() {
            let progress = progress_event_rx
                .borrow()
                .as_ref()
                .map(|event: &ImportProgressEvent| event.summary.clone());
            // Borrow has already been released at this point
            if let Some(progress) = progress {
                *media_tracker_progress.lock().await = MediaTrackerProgress::Importing(progress);
            }
        }
        log::debug!("Unwatching media tracker importing");
        *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
    });
    let response = websrv::spawn_blocking_write_task(
        &shared_connection_gatekeeper,
        move |mut pooled_connection| {
            abort_flag.store(false, Ordering::Relaxed);
            api::media::tracker::import_files::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
                &mut |progress_event| {
                    if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                        log::error!(
                            "Failed to send media tracker importing progress event: {:?}",
                            err.0
                        );
                    }
                },
                &abort_flag,
            )
        },
    )
    .await;
    if let Err(err) = watcher.await {
        log::error!("Failed to terminate media tracker importing progress watcher: {err}");
    }
    response.map(Json)
}

async fn media_tracker_post_collection_untrack(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::tracker::untrack_directories::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::media::tracker::untrack_directories::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn media_tracker_post_collection_find_untracked_files(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::tracker::find_untracked_files::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    let ApiState {
        rt,
        shared_connection_gatekeeper,
        abort_flag,
        media_tracker_progress,
    } = state;
    let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
    let watcher = rt.spawn(async move {
        *media_tracker_progress.lock().await =
            MediaTrackerProgress::FindingUntracked(Default::default());
        log::debug!("Watching media tracker finding untracked");
        while progress_event_rx.changed().await.is_ok() {
            let progress = progress_event_rx
                .borrow()
                .as_ref()
                .map(|event: &FindUntrackedProgressEvent| event.progress.clone());
            // Borrow has already been released at this point
            if let Some(progress) = progress {
                *media_tracker_progress.lock().await =
                    MediaTrackerProgress::FindingUntracked(progress);
            }
        }
        log::debug!("Unwatching media tracker finding untracked");
        *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
    });
    let response = websrv::spawn_blocking_read_task(
        &shared_connection_gatekeeper,
        move |mut pooled_connection| {
            abort_flag.store(false, Ordering::Relaxed);
            api::media::tracker::find_untracked_files::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
                &mut |progress_event: FindUntrackedProgressEvent| {
                    if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                        log::error!(
                            "Failed to send media tracker finding untracked progress event: {:?}",
                            err.0
                        );
                    }
                },
                &abort_flag,
            )
        },
    )
    .await;
    if let Err(err) = watcher.await {
        log::error!("Failed to terminate media tracker finding untracked progress watcher: {err}");
    }
    response.map(Json)
}

// Collected media sources

async fn collected_media_sources_relocate(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::source::relocate::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::media::source::relocate::handle_request(&mut pooled_connection, &uid, request_body)
        },
    )
    .await
    .map(Json)
}

async fn collected_media_sources_purge_orphaned(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::source::purge_orphaned::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::media::source::purge_orphaned::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn collected_media_sources_purge_untracked(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::media::source::purge_untracked::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::media::source::purge_untracked::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

// Collected tracks

async fn collected_tracks_resolve(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::track::resolve::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::resolve::handle_request(&mut pooled_connection, &uid, request_body)
        },
    )
    .await
    .map(Json)
}

async fn collected_tracks_search(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    QueryParams(query_params): QueryParams<api::track::search::QueryParams>,
    JsonBody(request_body): JsonBody<api::track::search::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::search::handle_request(
                &mut pooled_connection,
                &uid,
                query_params,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn collected_tracks_replace(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    QueryParams(query_params): QueryParams<api::track::replace::QueryParams>,
    JsonBody(request_body): JsonBody<api::track::replace::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::replace::handle_request(
                &mut pooled_connection,
                &uid,
                query_params,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn collected_tracks_import_and_replace(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    QueryParams(query_params): QueryParams<api::track::import_and_replace::QueryParams>,
    JsonBody(request_body): JsonBody<api::track::import_and_replace::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    let abort_flag = state.abort_flag;
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            abort_flag.store(false, Ordering::Relaxed);
            api::track::import_and_replace::handle_request(
                &mut pooled_connection,
                &uid,
                query_params,
                request_body,
                &abort_flag,
            )
        },
    )
    .await
    .map(Json)
}

async fn collected_tracks_find_unsynchronized(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    QueryParams(query_params): QueryParams<api::track::find_unsynchronized::QueryParams>,
    JsonBody(request_body): JsonBody<api::track::find_unsynchronized::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::find_unsynchronized::handle_request(
                &mut pooled_connection,
                &uid,
                query_params,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn collected_tracks_export_vfs(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::track::vfs::export_files::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::vfs::export_files::handle_request(
                &mut pooled_connection,
                &uid,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

// Tracks

async fn tracks_load_one(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<TrackUid>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::load_one::handle_request(&mut pooled_connection, &uid)
        },
    )
    .await
    .map(Json)
}

async fn tracks_load_many(
    State(state): State<ApiState>,
    JsonBody(request_body): JsonBody<api::track::load_many::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::load_many::handle_request(&mut pooled_connection, request_body)
        },
    )
    .await
    .map(Json)
}

async fn tracks_export_metadata(
    State(state): State<ApiState>,
    PathParam(track_uid): PathParam<TrackUid>,
    QueryParams(query_params): QueryParams<api::track::export_metadata::QueryParams>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::track::export_metadata::handle_request(
                &mut pooled_connection,
                &track_uid,
                query_params,
            )
        },
    )
    .await
    .map(Json)
}

// Playlists

async fn playlists_create(
    State(state): State<ApiState>,
    JsonBody(request_body): JsonBody<api::playlist::create::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::create::handle_request(&mut pooled_connection, None, request_body)
        },
    )
    .await
    .map(|response_body| (StatusCode::CREATED, Json(response_body)))
}

async fn playlists_load_one(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<PlaylistUid>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::load_one::handle_request(&mut pooled_connection, &uid)
        },
    )
    .await
    .map(Json)
}

async fn playlists_load_all(
    State(state): State<ApiState>,
    QueryParams(query_params): QueryParams<api::playlist::load_all::QueryParams>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::load_all::handle_request(&mut pooled_connection, None, query_params)
        },
    )
    .await
    .map(Json)
}

async fn playlists_update(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<PlaylistUid>,
    QueryParams(query_params): QueryParams<api::EntityRevQueryParams>,
    JsonBody(request_body): JsonBody<api::playlist::update::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::update::handle_request(
                &mut pooled_connection,
                uid,
                query_params,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

async fn playlists_delete(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<PlaylistUid>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::purge::handle_request(&mut pooled_connection, &uid)
        },
    )
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

async fn playlists_entries_patch(
    State(state): State<ApiState>,
    PathParam(uid): PathParam<PlaylistUid>,
    QueryParams(query_params): QueryParams<api::EntityRevQueryParams>,
    JsonBody(request_body): JsonBody<api::playlist::entries::patch::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::entries::patch::handle_request(
                &mut pooled_connection,
                uid,
                query_params,
                request_body,
            )
        },
    )
    .await
    .map(Json)
}

// Collected playlists

async fn collected_playlists_create(
    State(state): State<ApiState>,
    PathParam(collection_uid): PathParam<CollectionUid>,
    JsonBody(request_body): JsonBody<api::playlist::create::RequestBody>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::create::handle_request(
                &mut pooled_connection,
                Some(&collection_uid),
                request_body,
            )
        },
    )
    .await
    .map(|response_body| (StatusCode::CREATED, Json(response_body)))
}

async fn collected_playlists_load_all(
    State(state): State<ApiState>,
    PathParam(collection_uid): PathParam<CollectionUid>,
    QueryParams(query_params): QueryParams<api::playlist::load_all::QueryParams>,
) -> websrv::Result<impl IntoResponse> {
    let collection_filter = CollectionFilter {
        uid: Some(Cow::Owned(collection_uid)),
    };
    websrv::spawn_blocking_read_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| {
            api::playlist::load_all::handle_request(
                &mut pooled_connection,
                Some(collection_filter),
                query_params,
            )
        },
    )
    .await
    .map(Json)
}

// Storage

async fn storage_get_pending_tasks(State(state): State<ApiState>) -> impl IntoResponse {
    let PendingTasks { read, write } = state.shared_connection_gatekeeper.pending_tasks();
    Json(serde_json::json!({
        "read": read,
        "write": write,
    }))
}

async fn storage_post_abort_current_task(State(state): State<ApiState>) -> impl IntoResponse {
    state.abort_flag.store(true, Ordering::Relaxed);
    StatusCode::ACCEPTED
}

async fn storage_migrate_schema(
    State(state): State<ApiState>,
) -> websrv::Result<impl IntoResponse> {
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| uc::database::migrate_schema(&mut pooled_connection),
    )
    .await
    .map(|()| StatusCode::NO_CONTENT)
}

async fn storage_cleanse(
    State(state): State<ApiState>,
    QueryParams(query_params): QueryParams<CleanseDatabaseQueryParams>,
) -> websrv::Result<impl IntoResponse> {
    let CleanseDatabaseQueryParams { vacuum } = query_params;
    let vacuum_mode = if vacuum {
        Some(DEFAULT_VACUUM_MODE)
    } else {
        None
    };
    websrv::spawn_blocking_write_task(
        &state.shared_connection_gatekeeper,
        move |mut pooled_connection| cleanse_database(&mut pooled_connection, vacuum_mode),
    )
    .await
    .map(|()| StatusCode::NO_CONTENT)
}
