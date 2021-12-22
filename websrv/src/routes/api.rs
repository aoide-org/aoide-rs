// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::entity::EntityUid;

use aoide_core_api::media::tracker::Progress as MediaTrackerProgress;

use aoide_storage_sqlite::{
    cleanse_database,
    tokio::{DatabaseConnectionGatekeeper, PendingTasks},
};

use aoide_usecases::media::tracker::{
    find_untracked_files::ProgressEvent as FindUntrackedProgressEvent,
    import::ProgressEvent as ImportProgressEvent, scan::ProgressEvent as ScanProgressEvent,
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

    tracing::info!("Creating API routes");

    let path_param_uid = warp::path::param::<EntityUid>();

    let collections_path = warp::path("c");
    let tracks_path = warp::path("t");
    let playlists_path = warp::path("p");
    let media_source_path = warp::path("ms");
    let media_tracker_path = warp::path("mt");
    let storage_path = warp::path("storage");

    // Collections
    let collections_create = warp::post()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |request_body, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_write_task(&shared_connection_gatekeeper, move |pooled_connection, _abort_flag| {
                        uc_json::collection::create::handle_request(pooled_connection, request_body)
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
                            pooled_connection,
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
                        uc_json::collection::purge::handle_request(pooled_connection, &uid)
                    },
                )
                .await
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let collections_list = warp::get()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::query())
        .and(shared_connection_gatekeeper.clone())
        .and_then(
            move |query_params, shared_connection_gatekeeper: Arc<DatabaseConnectionGatekeeper>| async move {
                webapi::spawn_blocking_read_task(&shared_connection_gatekeeper,move |pooled_connection, _abort_flag| {
                        uc_json::collection::load_all::handle_request(pooled_connection, query_params)
                    })
                    .await
                    .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collections_get = warp::get()
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
                            pooled_connection,
                            &uid,
                            query_params,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collections_filters = collections_list
        .or(collections_get)
        .or(collections_create)
        .or(collections_update)
        .or(collections_delete);

    let collected_media_sources_relocate = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path("relocate-media-sources"))
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
                        uc_json::media::relocate_collected_sources::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );

    async fn reply_media_tracker_progress(
        media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>,
    ) -> Result<impl warp::Reply, Infallible> {
        let progress = media_tracker_progress.lock().await.clone();
        Ok(warp::reply::json(
            &aoide_core_api_json::media::tracker::Progress::from(progress),
        ))
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
                            pooled_connection,
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
        .and(warp::path("scan"))
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
                    tracing::debug!("Watching media tracker scanning");
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
                    tracing::debug!("Unwatching media tracker scanning");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::media::tracker::scan::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                            &mut |progress_event: ScanProgressEvent| {
                                if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                                    tracing::error!(
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
                    tracing::error!(
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
        .and(warp::path("import"))
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
                    tracing::debug!("Watching media tracker importing");
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
                    tracing::debug!("Unwatching media tracker importing");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = webapi::spawn_blocking_write_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::media::tracker::import::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                            &mut |progress_event| {
                                if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                                    tracing::error!(
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
                    tracing::error!(
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
        .and(warp::path("untrack"))
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
                        uc_json::media::tracker::untrack::handle_request(
                            pooled_connection,
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
                    tracing::debug!("Watching media tracker finding untracked");
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
                    tracing::debug!("Unwatching media tracker finding untracked");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = webapi::spawn_blocking_read_task(
                    &shared_connection_gatekeeper,
                    move |pooled_connection, abort_flag| {
                        uc_json::media::tracker::find_untracked_files::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                            &mut |progress_event: FindUntrackedProgressEvent| {
                                if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                                    tracing::error!(
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
                    tracing::error!(
                        "Failed to terminate media tracker finding untracked progress watcher: {}",
                        err
                    );
                }
                response.map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_post_collection_purge_untracked_sources = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("purge-untracked-sources"))
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
                        uc_json::media::tracker::purge_untracked_sources::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_filters = media_tracker_get_progress
        .or(media_tracker_post_collection_scan)
        .or(media_tracker_post_collection_import)
        .or(media_tracker_post_collection_untrack)
        .or(media_tracker_post_collection_find_untracked_files)
        .or(media_tracker_post_collection_purge_untracked_sources)
        .or(media_tracker_post_collection_query_status);

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
                            pooled_connection,
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
                            pooled_connection,
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
                            pooled_connection,
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
                            pooled_connection,
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
    let collected_tracks_filters = collected_tracks_resolve
        .or(collected_tracks_search)
        .or(collected_tracks_replace)
        .or(collected_tracks_import_and_replace);

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
                        uc_json::track::load_one::handle_request(pooled_connection, &uid)
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
                        uc_json::track::load_many::handle_request(pooled_connection, request_body)
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
                            pooled_connection,
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
                        uc_json::playlist::create_collected::handle_request(
                            pooled_connection,
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
                        uc_json::playlist::list_collected::handle_request(
                            pooled_connection,
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
                            pooled_connection,
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
                        uc_json::playlist::purge::handle_request(pooled_connection, &uid)
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
                        uc_json::playlist::patch_entries::handle_request(
                            pooled_connection,
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

    let media_import_track = warp::post()
        .and(media_source_path)
        .and(warp::path("import-track"))
        .and(warp::path::end())
        .and(warp::query())
        .and_then(|query_params| async move {
            webapi::after_blocking_task_finished(
                tokio::task::spawn_blocking(move || {
                    uc_json::media::import_track::handle_request(query_params)
                })
                .await,
            )
            .map(|response_body| warp::reply::json(&response_body))
        });

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
                        cleanse_database(&pooled_connection, vacuum)
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
        .or(collections_filters)
        .or(tracks_filters)
        .or(playlists_filters)
        .or(media_import_track)
        .or(collected_media_sources_relocate)
        .or(media_tracker_filters)
        .or(storage_filters)
        .boxed()
}