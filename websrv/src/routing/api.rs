// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::{convert::Infallible, sync::Arc};

use tokio::sync::{watch, Mutex};
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

#[cfg(feature = "with-schemars")]
use schemars::schema_for;

use aoide_core::entity::EntityUid;

use aoide_storage_sqlite::{
    cleanse_database,
    connection::pool::gatekeeper::{Gatekeeper as DatabaseConnectionGatekeeper, PendingTasks},
};

use aoide_usecases::media::tracker::{
    find_untracked_files::ProgressEvent as FindUntrackedProgressEvent,
    import_files::ProgressEvent as ImportProgressEvent,
    scan_directories::ProgressEvent as ScanProgressEvent, Progress as MediaTrackerProgress,
};

use aoide_websrv_api as webapi;

use aoide_usecases_sqlite as uc;

use aoide_usecases_sqlite_json as uc_json;

pub fn create_filters(
    shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>,
) -> BoxedFilter<(impl Reply,)> {
    let shared_connection_gatekeeper =
        warp::any().map(move || Arc::clone(&shared_connection_gatekeeper));

    let media_tracker_progress = Arc::new(Mutex::new(MediaTrackerProgress::Idle));
    let media_tracker_progress = warp::any().map(move || Arc::clone(&media_tracker_progress));

    log::info!("Creating API routes");

    let path_param_uid = warp::path::param::<EntityUid>();

    let collections_path = warp::path("c");
    let tracks_path = warp::path("t");
    let playlists_path = warp::path("p");
    let media_source_path = warp::path("ms");
    let media_tracker_path = warp::path("mt");
    let storage_path = warp::path("storage");

    #[cfg(feature = "with-schemars")]
    let schema_path = warp::path("schema");
    #[cfg(feature = "with-schemars")]
    let schema_get_path = schema_path.and(warp::path("get"));
    #[cfg(feature = "with-schemars")]
    let schema_post_path = schema_path.and(warp::path("post"));
    #[cfg(feature = "with-schemars")]
    let schema_put_path = schema_path.and(warp::path("put"));

    // Collections
    let collections_create = warp::post()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |request_body, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(&shared_connection_gatekeeper, move |pooled_connection, _abort_flag| {
                        uc_json::collection::create::handle_request(&*pooled_connection, request_body)
                    })
                    .await
                    .map(|response_body| {
                        warp::reply::with_status(
                            warp::reply::json(&response_body),
                            StatusCode::CREATED,
                        )
                    })
            },
        );
    #[cfg(feature = "with-schemars")]
    let collections_create_schema = warp::get()
        .and(schema_post_path)
        .and(collections_path)
        .and(warp::path::end())
        .map(|| {
            let request_schema = schema_for!(uc_json::collection::create::RequestBody);
            let response_schema = schema_for!(uc_json::collection::create::ResponseBody);
            let schema = serde_json::json!({
                "request": request_schema,
                "response": response_schema,
            });
            warp::reply::json(&schema)
        });

    let collections_update = warp::put()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::collection::update::handle_request(
                            &*pooled_connection,
                            uid,
                            query_params,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    #[cfg(feature = "with-schemars")]
    let collections_update_schema = warp::get()
        .and(schema_put_path)
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .map(|_uid| {
            let query_schema = schema_for!(uc_json::collection::update::QueryParams);
            let request_schema = schema_for!(uc_json::collection::update::RequestBody);
            let response_schema = schema_for!(uc_json::collection::update::ResponseBody);
            let schema = serde_json::json!({
                "query": query_schema,
                "request": request_schema,
                "response": response_schema,
            });
            warp::reply::json(&schema)
        });

    let collections_delete = warp::delete()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::collection::purge::handle_request(&*pooled_connection, &uid)
                    },
                )
                .await
                .map(|()| StatusCode::NO_CONTENT)
            },
        );

    let collections_load_all = warp::get()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::query())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |query_params, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(&shared_connection_gatekeeper,move |pooled_connection, _abort_flag| {
                        uc_json::collection::load_all::handle_request(&*pooled_connection, query_params)
                    })
                    .await
                    .map(|response_body| warp::reply::json(&response_body))
            },
        );
    #[cfg(feature = "with-schemars")]
    let collections_load_all_schema = warp::get()
        .and(schema_get_path)
        .and(collections_path)
        .and(warp::path::end())
        .map(|| {
            let query_schema = schema_for!(uc_json::collection::load_all::QueryParams);
            let response_schema = schema_for!(uc_json::collection::load_all::ResponseBody);
            let schema = serde_json::json!({
                "query": query_schema,
                "response": response_schema,
            });
            warp::reply::json(&schema)
        });

    let collections_load_one = warp::get()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::collection::load_one::handle_request(
                            &*pooled_connection,
                            &uid,
                            query_params,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    #[cfg(feature = "with-schemars")]
    let collections_load_one_schema = warp::get()
        .and(schema_get_path)
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .map(|_uid| {
            let query_schema = schema_for!(uc_json::collection::load_one::QueryParams);
            let response_schema = schema_for!(uc_json::collection::load_one::ResponseBody);
            let schema = serde_json::json!({
                "query": query_schema,
                "response": response_schema,
            });
            warp::reply::json(&schema)
        });

    let collections_load_all_kinds = warp::get()
        .and(collections_path)
        .and(warp::path("kinds"))
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::collection::load_all_kinds::handle_request(&*pooled_connection)
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    #[cfg(feature = "with-schemars")]
    let collections_load_all_kinds_schema = warp::get()
        .and(schema_get_path)
        .and(collections_path)
        .and(warp::path("kinds"))
        .and(warp::path::end())
        .map(|| {
            let response_schema = schema_for!(uc_json::collection::load_all_kinds::ResponseBody);
            let schema = serde_json::json!({
                "response": response_schema,
            });
            warp::reply::json(&schema)
        });

    let collections_filters = collections_load_all
        .or(collections_load_one)
        .or(collections_load_all_kinds)
        .or(collections_create)
        .or(collections_update)
        .or(collections_delete);

    #[cfg(feature = "with-schemars")]
    let collections_filters = collections_filters
        .or(collections_load_all_schema)
        .or(collections_load_one_schema)
        .or(collections_load_all_kinds_schema)
        .or(collections_create_schema)
        .or(collections_update_schema);

    async fn reply_media_tracker_progress(
        media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>,
    ) -> Result<impl warp::Reply, Infallible> {
        let progress = media_tracker_progress.lock().await.clone();
        Ok(warp::reply::json(&uc_json::media::tracker::Progress::from(
            progress,
        )))
    }

    let media_tracker_get_progress = warp::get()
        .and(media_tracker_path)
        .and(warp::path("progress"))
        .and(warp::path::end())
        .and(media_tracker_progress.clone())
        .and_then(
            |media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>| async move {
                reply_media_tracker_progress(media_tracker_progress).await
            },
        );
    let media_tracker_post_collection_query_status = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("query-status"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::media::tracker::query_status::handle_request(
                            &*pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_post_collection_scan = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("scan-directories"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and(media_tracker_progress.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>,
                  media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>| async move {
                let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
                let watcher = tokio::spawn(async move {
                    *media_tracker_progress.lock().await =
                        MediaTrackerProgress::Scanning(Default::default());
                    log::debug!("Watching media tracker scanning");
                    while progress_event_rx.changed().await.is_ok() {
                        let progress = progress_event_rx
                            .borrow()
                            .as_ref()
                            .map(|event: &ScanProgressEvent| event.progress.to_owned());
                        // Borrow has already been released at this point
                        if let Some(progress) = progress {
                            *media_tracker_progress.lock().await =
                                MediaTrackerProgress::Scanning(progress);
                        }
                    }
                    log::debug!("Unwatching media tracker scanning");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::media::tracker::scan_directories::handle_request(
                            &*pooled_connection,
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
                    log::error!(
                        "Failed to terminate media tracker scanning progress watcher: {}",
                        err
                    );
                }
                response.map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_post_collection_import = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("import-files"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and(media_tracker_progress.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>,
                  media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>| async move {
                let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
                let watcher = tokio::spawn(async move {
                    *media_tracker_progress.lock().await =
                        MediaTrackerProgress::Importing(Default::default());
                    log::debug!("Watching media tracker importing");
                    while progress_event_rx.changed().await.is_ok() {
                        let progress = progress_event_rx
                            .borrow()
                            .as_ref()
                            .map(|event: &ImportProgressEvent| event.summary.to_owned());
                        // Borrow has already been released at this point
                        if let Some(progress) = progress {
                            *media_tracker_progress.lock().await =
                                MediaTrackerProgress::Importing(progress);
                        }
                    }
                    log::debug!("Unwatching media tracker importing");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::media::tracker::import_files::handle_request(
                            &*pooled_connection,
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
                    log::error!(
                        "Failed to terminate media tracker importing progress watcher: {}",
                        err
                    );
                }
                response.map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_post_collection_untrack = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("untrack-directories"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::media::tracker::untrack_directories::handle_request(
                            &*pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_post_collection_find_untracked_files = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("find-untracked-files"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and(media_tracker_progress)
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>,
                  media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>| async move {
                let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
                let watcher = tokio::spawn(async move {
                    *media_tracker_progress.lock().await =
                        MediaTrackerProgress::FindingUntracked(Default::default());
                    log::debug!("Watching media tracker finding untracked");
                    while progress_event_rx.changed().await.is_ok() {
                        let progress = progress_event_rx
                            .borrow()
                            .as_ref()
                            .map(|event: &FindUntrackedProgressEvent| event.progress.to_owned());
                        // Borrow has already been released at this point
                        if let Some(progress) = progress {
                            *media_tracker_progress.lock().await =
                                MediaTrackerProgress::FindingUntracked(progress);
                        }
                    }
                    log::debug!("Unwatching media tracker finding untracked");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::media::tracker::find_untracked_files::handle_request(
                            &*pooled_connection,
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
                    log::error!(
                        "Failed to terminate media tracker finding untracked progress watcher: {}",
                        err
                    );
                }
                response.map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_filters = media_tracker_get_progress
        .or(media_tracker_post_collection_scan)
        .or(media_tracker_post_collection_import)
        .or(media_tracker_post_collection_untrack)
        .or(media_tracker_post_collection_find_untracked_files)
        .or(media_tracker_post_collection_query_status);

    // TODO: Add OpenAPI docs for all collected media source requests
    let collected_media_sources_relocate = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_source_path)
        .and(warp::path("relocate"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::media::source::relocate::handle_request(
                            &*pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_media_sources_purge_orphaned = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_source_path)
        .and(warp::path("purge-orphaned"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::media::source::purge_orphaned::handle_request(
                            &*pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_media_sources_purge_untracked = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_source_path)
        .and(warp::path("purge-untracked"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::media::source::purge_untracked::handle_request(
                            &*pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_media_sources_filters = collected_media_sources_relocate
        .or(collected_media_sources_purge_orphaned)
        .or(collected_media_sources_purge_untracked);

    let collected_tracks_resolve = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("resolve"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::track::resolve::handle_request(
                            &*pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_search = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("search"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::track::search::handle_request(
                            &*pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_replace = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("replace"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::track::replace::handle_request(
                            &*pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_import_and_replace = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("import-and-replace"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::track::import_and_replace::handle_request(
                            &*pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                            &abort_flag,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_find_unsynchronized = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("find-unsynchronized"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::track::find_unsynchronized::handle_request(
                            &*pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_filters = collected_tracks_resolve
        .or(collected_tracks_search)
        .or(collected_tracks_replace)
        .or(collected_tracks_import_and_replace)
        .or(collected_tracks_find_unsynchronized);

    // Tracks
    let tracks_load_one = warp::get()
        .and(tracks_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::track::load_one::handle_request(&*pooled_connection, &uid)
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_load_many = warp::post()
        .and(tracks_path)
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |request_body, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,move |pooled_connection, _abort_flag| {
                        uc_json::track::load_many::handle_request(&*pooled_connection, request_body)
                    })
                    .await
                    .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_export_metadata = warp::post()
        .and(tracks_path)
        .and(path_param_uid)
        .and(warp::path("export-metadata"))
        .and(warp::path::end())
        .and(warp::query())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |track_uid,
                  query_params,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::track::export_metadata::handle_request(
                            &*pooled_connection,
                            &track_uid,
                            query_params,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_filters = tracks_load_many
        .or(tracks_load_one)
        .or(tracks_export_metadata);

    let collected_playlists_create = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(playlists_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |collection_uid,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::playlist::create::handle_request(
                            &*pooled_connection,
                            &collection_uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::CREATED)
                })
            },
        );
    let collected_playlists_list = warp::get()
        .and(collections_path)
        .and(path_param_uid)
        .and(playlists_path)
        .and(warp::path::end())
        .and(warp::query())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |collection_uid,
                  query_params,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::playlist::load::handle_request(
                            &*pooled_connection,
                            &collection_uid,
                            query_params,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_playlists_filters = collected_playlists_list.or(collected_playlists_create);

    let playlists_update = warp::put()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::playlist::update::handle_request(
                            &*pooled_connection,
                            uid,
                            query_params,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let playlists_delete = warp::delete()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::playlist::purge::handle_request(&*pooled_connection, &uid)
                    },
                )
                .await
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let playlists_entries_patch = warp::patch()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path("entries"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc_json::playlist::entries::patch::handle_request(
                            &*pooled_connection,
                            uid,
                            query_params,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let playlists_filters = playlists_update
        .or(playlists_delete)
        .or(playlists_entries_patch);

    // Storage
    let storage_get_pending_tasks = warp::get()
        .and(storage_path)
        .and(warp::path("pending-tasks"))
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .map(
            |shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| {
                let PendingTasks { read, write } = shared_connection_gatekeeper.pending_tasks();
                warp::reply::json(&serde_json::json!({
                    "read": read,
                    "write": write,
                }))
            },
        );
    let storage_post_abort_current_task = warp::post()
        .and(storage_path)
        .and(warp::path("abort-current-task"))
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .map(
            |shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| {
                shared_connection_gatekeeper.abort_current_task();
                StatusCode::ACCEPTED
            },
        );
    let storage_migrate_schema = warp::post()
        .and(storage_path)
        .and(warp::path("migrate-schema"))
        .and(warp::path::end())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        uc::database::migrate_schema(&pooled_connection)
                    },
                )
                .await
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    // TODO: Move into separate request handler
    #[derive(serde::Deserialize)]
    struct CleanseDatabaseQueryParams {
        vacuum: bool,
    }
    let storage_cleanse = warp::post()
        .and(storage_path)
        .and(warp::path("cleanse"))
        .and(warp::path::end())
        .and(warp::query())
        .and(shared_connection_gatekeeper)
        .and_then(
            move |query_params, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                let CleanseDatabaseQueryParams { vacuum } = query_params;
                webapi::spawn_blocking_write_task(&shared_connection_gatekeeper,
                    move |pooled_connection, _abort_flag| {
                        cleanse_database(&*pooled_connection, vacuum)
                    })
                    .await
                    .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let storage_filters = storage_get_pending_tasks
        .or(storage_post_abort_current_task)
        .or(storage_migrate_schema)
        .or(storage_cleanse);

    collected_tracks_filters
        .or(collected_playlists_filters)
        .or(collected_media_sources_filters)
        .or(collections_filters)
        .or(tracks_filters)
        .or(playlists_filters)
        .or(media_tracker_filters)
        .or(storage_filters)
        .boxed()
}
