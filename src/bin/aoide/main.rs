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

#![warn(rust_2018_idioms)]

mod env;

use aoide::{
    api::web::{collections, handle_rejection, media, playlists, reject_on_error, tracks, Error},
    usecases as uc, *,
};

use aoide_core::entity::EntityUid;

use futures::future::{join, FutureExt};
use std::{
    collections::HashMap,
    env::current_exe,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};
use tokio::{sync::mpsc, sync::RwLock, time::sleep};
use warp::{http::StatusCode, Filter};

///////////////////////////////////////////////////////////////////////

const WEB_SERVER_LISTENING_DELAY: Duration = Duration::from_millis(250);

static INDEX_HTML: &str = include_str!("../../../resources/index.html");
static OPENAPI_YAML: &str = include_str!("../../../resources/openapi.yaml");

fn create_connection_pool(
    database_url: &str,
    max_size: u32,
) -> Result<SqliteConnectionPool, Error> {
    log::info!("Creating SQLite connection pool");
    let manager = SqliteConnectionManager::new(database_url);
    let pool = SqliteConnectionPool::builder()
        .max_size(max_size)
        .build(manager)?;
    Ok(pool)
}

// Let only a single writer at any time get access to the
// connection pool to prevent both synchronous locking when
// obtaining a connection and timeouts when concurrently
// trying to execute write operations on the shared SQLite
// database.
type GuardedConnectionPool = Arc<RwLock<SqliteConnectionPool>>;

const DB_CONNECTION_READ_GUARD_TIMEOUT: tokio::time::Duration =
    tokio::time::Duration::from_secs(10);
const DB_CONNECTION_WRITE_GUARD_TIMEOUT: tokio::time::Duration =
    tokio::time::Duration::from_secs(30);

async fn spawn_blocking_database_read_task<H, R>(
    guarded_connection_pool: GuardedConnectionPool,
    connection_handler: H,
) -> Result<R, Error>
where
    H: FnOnce(SqlitePooledConnection) -> Result<R, Error> + Send + 'static,
    R: Send + 'static,
{
    let timeout = tokio::time::sleep(DB_CONNECTION_READ_GUARD_TIMEOUT);
    tokio::pin!(timeout);
    tokio::select! {
        _ = &mut timeout => Err(Error::Timeout {reason: "database is locked".to_string() }),
        guard = guarded_connection_pool.read() => {
            let connection = guard.get()?;
            return tokio::task::spawn_blocking(move || connection_handler(connection)).await?
        },
    }
}

async fn spawn_blocking_database_write_task<H, R>(
    guarded_connection_pool: GuardedConnectionPool,
    connection_handler: H,
) -> Result<R, Error>
where
    H: FnOnce(SqlitePooledConnection) -> Result<R, Error> + Send + 'static,
    R: Send + 'static,
{
    let timeout = tokio::time::sleep(DB_CONNECTION_WRITE_GUARD_TIMEOUT);
    tokio::pin!(timeout);
    tokio::select! {
        _ = &mut timeout => Err(Error::Timeout {reason: "database is locked".to_string() }),
        guard = guarded_connection_pool.write() => {
            let connection = guard.get()?;
            return tokio::task::spawn_blocking(move || connection_handler(connection)).await?
        },
    }
}

static SCAN_MEDIA_DIRECTORIES_ABORT_FLAG: AtomicBool = AtomicBool::new(false);

#[tokio::main]
pub async fn main() -> Result<(), Error> {
    let started_at = chrono::Utc::now();

    env::init_environment();

    let log_level = env::init_logging();

    if let Ok(exe_path) = current_exe() {
        log::info!("Executable: {}", exe_path.display());
    }
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let endpoint_addr = env::parse_endpoint_addr();
    log::info!("Endpoint address: {}", endpoint_addr);

    let database_url = env::parse_database_url();
    log::info!("Database URL: {}", database_url);

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    let database_connection_pool_size = env::parse_database_connection_pool_size();
    let connection_pool = create_connection_pool(&database_url, database_connection_pool_size)
        .expect("Failed to create database connection pool");

    uc::database::initialize(&*connection_pool.get()?).expect("Failed to initialize database");
    uc::database::migrate_schema(&*connection_pool.get()?)
        .expect("Failed to migrate database schema");

    // Readers and writers are distinguished by an asynchronous RwLock
    // guard. This lock has to be acquired before requesting a connection
    // from the pool. Requesting a pooled connection may block the current
    // thread and has to be done in a spawned thread to prevent locking of
    // executor threads!
    let guarded_connection_pool = Arc::new(RwLock::new(connection_pool));

    log::info!("Creating service routes");

    let guarded_connection_pool = warp::any().map(move || guarded_connection_pool.clone());

    // POST /shutdown
    let (server_shutdown_tx, mut server_shutdown_rx) = mpsc::unbounded_channel::<()>();
    let shutdown_filter = warp::post()
        .and(warp::path("shutdown"))
        .and(warp::path::end())
        .map(move || {
            server_shutdown_tx
                .send(())
                .map(|()| StatusCode::ACCEPTED)
                .or_else(|_| {
                    log::warn!("Failed to forward shutdown request");
                    Ok(StatusCode::BAD_GATEWAY)
                })
        });

    // GET /about
    let about_filter = warp::get()
        .and(warp::path("about"))
        .and(warp::path::end())
        .map(move || {
            warp::reply::json(&serde_json::json!({
            "name": env!("CARGO_PKG_NAME"),
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "version": env!("CARGO_PKG_VERSION"),
            "instance": {
                "startedAt": started_at,
                "environment": {
                    "vars": std::env::vars().fold(HashMap::new(), |mut vars, (key, val)| {
                        debug_assert!(!vars.contains_key(&key));
                        vars.insert(key, val);
                        vars}),
                    "currentWorkingDirectory": std::env::current_dir().unwrap_or_default(),
                },
                "logging": {
                    "level": log_level.to_string().to_lowercase(),
                },
                "networking": {
                    "endpointAddress": endpoint_addr,
                },
                "database": {
                    "url": database_url,
                    "connectionPoolSize": database_connection_pool_size,
                }
            }
            }))
        });

    let path_param_uid = warp::path::param::<EntityUid>();

    let collections_path = warp::path("c");
    let tracks_path = warp::path("t");
    let playlists_path = warp::path("p");
    let media_path = warp::path("m");
    let media_dir_tracker_path = warp::path("media-dir-tracker");
    let storage_path = warp::path("storage");

    // Collections
    let collections_create = warp::post()
        .and(collections_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            |request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        collections::create::handle_request(pooled_connection, request_body)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| {
                    warp::reply::with_status(warp::reply::json(&response_body), StatusCode::CREATED)
                })
            },
        );
    let collections_update =
        warp::put()
            .and(collections_path)
            .and(path_param_uid)
            .and(warp::path::end())
            .and(warp::query())
            .and(warp::body::json())
            .and(guarded_connection_pool.clone())
            .and_then(
                |uid,
                 query_params,
                 request_body,
                 guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_write_task(
                        guarded_connection_pool,
                        move |pooled_connection| {
                            collections::update::handle_request(
                                pooled_connection,
                                uid,
                                query_params,
                                request_body,
                            )
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
            |uid, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        collections::delete::handle_request(pooled_connection, &uid)
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
            |query_params, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        collections::load_all::handle_request(pooled_connection, query_params)
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
            |uid, query_params, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        collections::load_one::handle_request(pooled_connection, &uid, query_params)
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
            |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        media::relocate_collected_sources::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );

    let media_dir_tracker_aggregate_status = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_dir_tracker_path)
        .and(warp::path("aggregate-status"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        media::digest_directories_aggregate_status::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_dir_tracker_scan_directories = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(media_dir_tracker_path)
        .and(warp::path("scan"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        // Reset abort flag
                        SCAN_MEDIA_DIRECTORIES_ABORT_FLAG.store(false, Ordering::Relaxed);
                        media::digest_directories::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                            &SCAN_MEDIA_DIRECTORIES_ABORT_FLAG,
                        )
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let media_dir_tracker_scan_directories_abort = warp::post()
        .and(media_dir_tracker_path)
        .and(warp::path("scan"))
        .and(warp::path("abort"))
        .and(warp::path::end())
        .map(|| {
            SCAN_MEDIA_DIRECTORIES_ABORT_FLAG.store(true, Ordering::Relaxed);
            StatusCode::ACCEPTED
        });

    let collected_tracks_resolve = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(tracks_path)
        .and(warp::path("resolve"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        tracks::resolve_collected::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_tracks_search =
        warp::post()
            .and(collections_path)
            .and(path_param_uid)
            .and(tracks_path)
            .and(warp::path("search"))
            .and(warp::path::end())
            .and(warp::query())
            .and(warp::body::json())
            .and(guarded_connection_pool.clone())
            .and_then(
                |uid,
                 query_params,
                 request_body,
                 guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_read_task(
                        guarded_connection_pool,
                        move |pooled_connection| {
                            tracks::search_collected::handle_request(
                                pooled_connection,
                                &uid,
                                query_params,
                                request_body,
                            )
                        },
                    )
                    .await
                    .map_err(reject_on_error)
                    .map(|response_body| warp::reply::json(&response_body))
                },
            );
    let collected_tracks_replace =
        warp::post()
            .and(collections_path)
            .and(path_param_uid)
            .and(tracks_path)
            .and(warp::path("replace"))
            .and(warp::path::end())
            .and(warp::query())
            .and(warp::body::json())
            .and(guarded_connection_pool.clone())
            .and_then(
                |uid,
                 query_params,
                 request_body,
                 guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_write_task(
                        guarded_connection_pool,
                        move |pooled_connection| {
                            tracks::replace_collected::handle_request(
                                pooled_connection,
                                &uid,
                                query_params,
                                request_body,
                            )
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
            |uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        tracks::purge_collected::handle_request(
                            pooled_connection,
                            &uid,
                            request_body,
                        )
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
        .or(collected_tracks_purge);

    // Tracks
    let tracks_load_one = warp::get()
        .and(tracks_path)
        .and(path_param_uid)
        .and(warp::path::end())
        .and(guarded_connection_pool.clone())
        .and_then(
            |uid, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        tracks::load_one::handle_request(pooled_connection, &uid)
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
            |request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        tracks::load_many::handle_request(pooled_connection, request_body)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let tracks_filters = tracks_load_many.or(tracks_load_one);

    let collected_playlists_create = warp::post()
        .and(collections_path)
        .and(path_param_uid)
        .and(playlists_path)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(guarded_connection_pool.clone())
        .and_then(
            |collection_uid, request_body, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        playlists::create_collected::handle_request(pooled_connection, &collection_uid, request_body)
                    },
                )
                .await
                .map_err(reject_on_error)
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
        .and(guarded_connection_pool.clone())
        .and_then(
            |collection_uid, query_params, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_read_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        playlists::list_collected::handle_request(pooled_connection, &collection_uid,
                            query_params)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
            },
        );
    let collected_playlists_filters = collected_playlists_list.or(collected_playlists_create);

    let playlists_update =
        warp::put()
            .and(playlists_path)
            .and(path_param_uid)
            .and(warp::path::end())
            .and(warp::query())
            .and(warp::body::json())
            .and(guarded_connection_pool.clone())
            .and_then(
                |uid,
                 query_params,
                 request_body,
                 guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_write_task(
                        guarded_connection_pool,
                        move |pooled_connection| {
                            playlists::update::handle_request(
                                pooled_connection,
                                uid,
                                query_params,
                                request_body,
                            )
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
            |uid, guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(
                    guarded_connection_pool,
                    move |pooled_connection| {
                        playlists::delete::handle_request(pooled_connection, &uid)
                    },
                )
                .await
                .map_err(reject_on_error)
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let playlists_entries_patch =
        warp::patch()
            .and(playlists_path)
            .and(path_param_uid)
            .and(warp::path("entries"))
            .and(warp::path::end())
            .and(warp::query())
            .and(warp::body::json())
            .and(guarded_connection_pool.clone())
            .and_then(
                |uid,
                 query_params,
                 request_body,
                 guarded_connection_pool: GuardedConnectionPool| async move {
                    spawn_blocking_database_write_task(
                        guarded_connection_pool,
                        move |pooled_connection| {
                            playlists::patch_entries::handle_request(
                                pooled_connection,
                                uid,
                                query_params,
                                request_body,
                            )
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
            tokio::task::spawn_blocking(move || media::import_track::handle_request(query_params))
                .await
                .map_err(reject_on_error)? // JoinError
                .map_err(reject_on_error)
                .map(|response_body| warp::reply::json(&response_body))
        });

    // Storage
    let storage_cleanse = warp::post()
        .and(storage_path)
        .and(warp::path("cleanse"))
        .and(warp::path::end())
        .and(guarded_connection_pool.clone())
        .and_then(
            |guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(guarded_connection_pool, |pooled_connection| {
                    Ok(uc::database::cleanse(&pooled_connection)?)
                })
                .await
                .map_err(reject_on_error)
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let storage_optimize = warp::post()
        .and(storage_path)
        .and(warp::path("optimize"))
        .and(warp::path::end())
        .and(guarded_connection_pool.clone())
        .and_then(
            |guarded_connection_pool: GuardedConnectionPool| async move {
                spawn_blocking_database_write_task(guarded_connection_pool, |pooled_connection| {
                    Ok(uc::database::optimize(&pooled_connection)?)
                })
                .await
                .map_err(reject_on_error)
                .map(|()| StatusCode::NO_CONTENT)
            },
        );
    let storage_filters = storage_cleanse.or(storage_optimize);

    // Static content
    let index_html = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let openapi_yaml = warp::path("openapi.yaml").map(|| {
        warp::reply::with_header(
            OPENAPI_YAML,
            "Content-Type",
            "application/x-yaml;charset=utf-8",
        )
    });
    let static_filters = index_html.or(openapi_yaml);

    log::info!("Initializing server");

    let server = warp::serve(
        collected_tracks_filters
            .or(collected_playlists_filters)
            .or(collections_filters)
            .or(tracks_filters)
            .or(playlists_filters)
            .or(media_import_track) // undocumented
            .or(media_dir_tracker_scan_directories)
            .or(media_dir_tracker_aggregate_status)
            .or(collected_media_sources_relocate)
            .or(media_dir_tracker_scan_directories_abort)
            .or(storage_filters)
            .or(static_filters)
            .or(shutdown_filter)
            .or(about_filter)
            .with(warp::cors().allow_any_origin())
            .recover(handle_rejection),
    );

    log::info!("Starting");

    let (socket_addr, server_listener) =
        server.bind_with_graceful_shutdown(endpoint_addr, async move {
            server_shutdown_rx.recv().await;
            SCAN_MEDIA_DIRECTORIES_ABORT_FLAG.store(true, Ordering::Relaxed);
            log::info!("Stopping");
        });

    let server_listening = async move {
        // Give the server some time to become ready and start listening
        // before announcing the actual endpoint address, i.e. when using
        // an ephemeral port. The delay might need to be tuned depending
        // on how long the startup actually takes. Unfortunately warp does
        // not provide any signal when the server has started listening.
        sleep(WEB_SERVER_LISTENING_DELAY).await;

        // -> stderr
        log::info!("Listening on {}", socket_addr);
        // -> stdout
        println!("{}", socket_addr);
    };

    join(server_listener, server_listening).map(drop).await;
    log::info!("Stopped");

    Ok(())
}
