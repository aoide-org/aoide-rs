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

use std::{
    convert::Infallible,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use tokio::sync::{watch, Mutex};
use warp::{filters::BoxedFilter, http::StatusCode, Filter, Reply};

use aoide_usecases::media::tracker::scan::ProgressEvent as ScanProgressEvent;

use aoide_core_ext::media::tracker::import::Summary as ImportProgressSummary;

use aoide_websrv::api::{reject_on_error, Error};

use aoide_jsonapi_sqlite as api;

use aoide_usecases_sqlite as uc;

use aoide_core::entity::EntityUid;

use aoide_core_ext::media::tracker::Progress as MediaTrackerProgress;

use crate::GuardedConnectionPool;

const DB_CONNECTION_READ_GUARD_TIMEOUT: tokio::time::Duration =
    tokio::time::Duration::from_secs(10);
const DB_CONNECTION_WRITE_GUARD_TIMEOUT: tokio::time::Duration =
    tokio::time::Duration::from_secs(30);

async fn spawn_blocking_database_read_task<H, R>(
    guarded_connection_pool: GuardedConnectionPool,
    media_tracker_abort_flag: &'static AtomicBool,
    connection_handler: H,
) -> Result<R, Error>
where
    H: FnOnce(uc::SqlitePooledConnection) -> Result<R, Error> + Send + 'static,
    R: Send + 'static,
{
    // Implicitly abort any running batch operation to prevent starving
    media_tracker_abort_flag.store(false, Ordering::Relaxed);
    let timeout = tokio::time::sleep(DB_CONNECTION_READ_GUARD_TIMEOUT);
    tokio::pin!(timeout);
    tokio::select! {
        _ = &mut timeout => Err(Error::Timeout {reason: "database is locked".to_string() }),
        guard = guarded_connection_pool.read() => {
            let connection = uc::database::get_pooled_connection(&*guard)?;
            return tokio::task::spawn_blocking(move || connection_handler(connection)).await?
        },
    }
}

async fn spawn_blocking_database_write_task<H, R>(
    guarded_connection_pool: GuardedConnectionPool,
    media_tracker_abort_flag: &'static AtomicBool,
    connection_handler: H,
) -> Result<R, Error>
where
    H: FnOnce(uc::SqlitePooledConnection) -> Result<R, Error> + Send + 'static,
    R: Send + 'static,
{
    // Implicitly abort any running batch operation to prevent starving
    media_tracker_abort_flag.store(false, Ordering::Relaxed);
    let timeout = tokio::time::sleep(DB_CONNECTION_WRITE_GUARD_TIMEOUT);
    tokio::pin!(timeout);
    tokio::select! {
        _ = &mut timeout => Err(Error::Timeout {reason: "database is locked".to_string() }),
        guard = guarded_connection_pool.write() => {
            let connection = uc::database::get_pooled_connection(&*guard)?;
            return tokio::task::spawn_blocking(move || connection_handler(connection)).await?
        },
    }
}

pub fn create_filters(
    guarded_connection_pool: GuardedConnectionPool,
    media_tracker_abort_flag: &'static AtomicBool,
) -> BoxedFilter<(impl Reply,)> {
    let guarded_connection_pool = warp::any().map(move || Arc::clone(&guarded_connection_pool));

    let media_tracker_progress = Arc::new(Mutex::new(MediaTrackerProgress::Idle));
    let media_tracker_progress = warp::any().map(move || Arc::clone(&media_tracker_progress));

    tracing::info!("Creating API routes");

    let path_param_uid = warp::path::param::<EntityUid>();

    let collections_path = warp::path("c");
    let tracks_path = warp::path("t");
    let playlists_path = warp::path("p");
    let media_path = warp::path("m");
    let media_tracker_path = warp::path("mt");
    let storage_path = warp::path("storage");

    // Collections
    let collections_create = warp::post()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::collection::create::handle_request(pooled_connection, request_body)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::CREATED)
                })
            },
        );
    let collections_update = warp::put()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::collection::update::handle_request(
                            pooled_connection,
                            uid,
                            query_params,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collections_delete = warp::delete()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::collection::purge::handle_request(pooled_connection, &uid)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let collections_list = warp::get()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::query())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |query_params, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::collection::load_all::handle_request(pooled_connection, query_params)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collections_get = warp::get()
        .and(collections_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, query_params, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::collection::load_one::handle_request(
                            pooled_connection,
                            &uid,
                            query_params,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::media::relocate_collected_sources::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );

    async fn reply_media_tracker_progress(
        media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>,
    ) -> Result<impl warp::Reply, Infallible> {
        let progress = media_tracker_progress.lock().await.clone();
        Ok(warp::reply::json(
            &aoide_core_ext_serde::media::tracker::Progress::from(progress),
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
    let media_tracker_post_abort = warp::post()
        .and(media_tracker_path)
        .and(warp::path("abort"))
        .and(warp::path::end())
        .map(|| {
            media_tracker_abort_flag.store(true, Ordering::Relaxed);
            StatusCode::ACCEPTED
        });
    let media_tracker_post_collection_query_status = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("query-status"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::media::tracker::query_status::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and(media_tracker_progress.clone())
        .and_then(
            move |uid,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool,
                  media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>| async move {
                let (progress_event_tx, mut progress_event_rx) = watch::channel(None);
                let watcher = tokio::spawn(async move {
                    *media_tracker_progress.lock().await =
                        MediaTrackerProgress::Scanning(Default::default());
                    tracing::debug!("Watching media tracker scanning");
                    while progress_event_rx.changed().await.is_ok() {
                        let progress = progress_event_rx.borrow().as_ref().map(
                            |event: &aoide_usecases::media::tracker::scan::ProgressEvent| {
                                event.progress.to_owned()
                            },
                        );
                        // Borrow has already been released at this point
                        if let Some(progress) = progress {
                            *media_tracker_progress.lock().await =
                                MediaTrackerProgress::Scanning(progress);
                        }
                    }
                    tracing::debug!("Unwatching media tracker scanning");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::media::tracker::scan::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                            &mut |progress_event: ScanProgressEvent| {
                                if let Err(err) = progress_event_tx.send(Some(progress_event)) {
                                    tracing::error!(
                                        "Failed to send scan progress event: {:?}",
                                        err.0
                                    );
                                }
                            },
                            media_tracker_abort_flag,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body));
                if let Err(err) = watcher.await {
                    tracing::error!(
                        "Failed to terminate media tracker scanning progress watcher: {}",
                        err
                    );
                }
                response
            },
        );
    let media_tracker_post_collection_import = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("import"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and(media_tracker_progress)
        .and_then(
            move |uid,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool,
                  media_tracker_progress: Arc<Mutex<MediaTrackerProgress>>| async move {
                let (progress_summary_tx, mut progress_summary_rx) =
                    watch::channel(aoide_core_ext::media::tracker::import::Summary::default());
                let watcher = tokio::spawn(async move {
                    *media_tracker_progress.lock().await =
                        MediaTrackerProgress::Importing(Default::default());
                    tracing::debug!("Watching media tracker importing");
                    while progress_summary_rx.changed().await.is_ok() {
                        let progress = progress_summary_rx.borrow().to_owned();
                        // Borrow has already been released at this point
                        *media_tracker_progress.lock().await =
                            MediaTrackerProgress::Importing(progress);
                    }
                    tracing::debug!("Unwatching media tracker importing");
                    *media_tracker_progress.lock().await = MediaTrackerProgress::Idle;
                });
                let response = spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::media::tracker::import::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                            &mut |progress_summary: &ImportProgressSummary| {
                                if let Err(err) =
                                    progress_summary_tx.send(progress_summary.to_owned())
                                {
                                    tracing::error!(
                                        "Failed to send import progress summary: {:?}",
                                        err.0
                                    );
                                }
                            },
                            media_tracker_abort_flag,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body));
                if let Err(err) = watcher.await {
                    tracing::error!(
                        "Failed to terminate media tracker importing progress watcher: {}",
                        err
                    );
                }
                response
            },
        );
    let media_tracker_post_collection_untrack = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("untrack"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::media::tracker::untrack::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_post_collection_purge_untracked = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_tracker_path)
        .and(warp::path("purge-untracked"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::purge_untracked_media::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_tracker_filters = media_tracker_get_progress
        .or(media_tracker_post_abort)
        .or(media_tracker_post_collection_scan)
        .or(media_tracker_post_collection_import)
        .or(media_tracker_post_collection_untrack)
        .or(media_tracker_post_collection_purge_untracked)
        .or(media_tracker_post_collection_query_status);

    let collected_tracks_resolve = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("resolve"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::resolve::handle_request(pooled_connection, &uid, request_body)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::search::handle_request(
                            pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::replace::handle_request(
                            pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                let abort_flag = AtomicBool::new(false);
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::import_and_replace::handle_request(
                            pooled_connection,
                            &uid,
                            query_params,
                            request_body,
                            &abort_flag,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_purge = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("purge"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::purge_media::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_filters = collected_tracks_resolve
        .or(collected_tracks_search)
        .or(collected_tracks_replace)
        .or(collected_tracks_import_and_replace)
        .or(collected_tracks_purge);

    // Tracks
    let tracks_load_one = warp::get()
        .and(tracks_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::load_one::handle_request(pooled_connection, &uid)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_load_many = warp::post()
        .and(tracks_path)
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::load_many::handle_request(pooled_connection, request_body)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_export_metadata = warp::post()
        .and(tracks_path)
        .and(path_param_uid)
        .and(warp::path("export-metadata"))
        .and(warp::path::end())
        .and(warp::query())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |
                  track_uid,
                  query_params,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::track::export_metadata::handle_request(
                            pooled_connection,
                            &track_uid,
                            query_params,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_filters = tracks_load_many
        .or(tracks_load_one)
        .or(tracks_export_metadata);

    let collected_playlists_create =
        warp::post()
            .and(collections_path)
            .and(path_param_uid)
            .and(playlists_path)
            .and(warp::path::end())
            .and(warp::body::json())
            .and(guarded_connection_pool.clone())
            .and_then(
                move |collection_uid,
                      request_body,
                      guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_write_task(
                        guarded_connection_pool,
                        media_tracker_abort_flag,
                        move |pooled_connection| {
                            api::playlist::create_collected::handle_request(
                                pooled_connection,
                                &collection_uid,
                                request_body,
                            )
                            .map_err(Into::into)
                        },
                    )
                    .await
                    .map_err(reject_on_error)
                    .map(|response_body| {
                        warp::reply::with_status(
                            warp::reply::json(&response_body),
                            StatusCode::CREATED,
                        )
                    })
                },
            );
    let collected_playlists_list =
        warp::get()
            .and(collections_path)
            .and(path_param_uid)
            .and(playlists_path)
            .and(warp::path::end())
            .and(warp::query())
            .and(guarded_connection_pool.clone())
            .and_then(
                move |collection_uid,
                      query_params,
                      guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_read_task(
                        guarded_connection_pool,
                        media_tracker_abort_flag,
                        move |pooled_connection| {
                            api::playlist::list_collected::handle_request(
                                pooled_connection,
                                &collection_uid,
                                query_params,
                            )
                            .map_err(Into::into)
                        },
                    )
                    .await
                    .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::playlist::update::handle_request(
                            pooled_connection,
                            uid,
                            query_params,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let playlists_delete = warp::delete()
        .and(playlists_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::playlist::purge::handle_request(pooled_connection, &uid)
                            .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            move |uid,
                  query_params,
                  request_body,
                  guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        api::playlist::patch_entries::handle_request(
                            pooled_connection,
                            uid,
                            query_params,
                            request_body,
                        )
                        .map_err(Into::into)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let playlists_filters = playlists_update
        .or(playlists_delete)
        .or(playlists_entries_patch);

    let media_import_track = warp::post()
        .and(media_path)
        .and(warp::path("import-track"))
        .and(warp::path::end())
        .and(warp::query())
        .and_then(|query_params| async move {
            tokio::task::spawn_blocking(move || {
                api::media::import_track::handle_request(query_params)
            })
            .await
            .map_err(reject_on_error)? // JoinError
            .map_err(reject_on_error)
            .map(|response_body| warp::reply::json(&response_body))
        });

    // Storage
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
        .and(guarded_connection_pool)
        .and_then(
            move |query_params, guarded_connection_pool: GuardedConnectionPool| async move {
                let CleanseDatabaseQueryParams { vacuum } = query_params;
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    media_tracker_abort_flag,
                    move |pooled_connection| {
                        Ok(uc::database::cleanse(&pooled_connection, vacuum)
                            .map_err(api::Error::from)?)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let storage_filters = storage_cleanse;

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
