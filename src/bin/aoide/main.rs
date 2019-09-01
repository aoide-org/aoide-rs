// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#![recursion_limit = "128"]

mod cli;
mod env;

use aoide::{
    api::web::{collections::*, tracks::*},
    *,
};

use aoide_core::collection::Collection;

use aoide_repo_sqlite::track::util::RepositoryHelper as TrackRepositoryHelper;

use clap::App;
use diesel::{prelude::*, sql_query};
use failure::Error;
use futures::{future, Future, Stream};
use std::{
    env::current_exe,
    net::SocketAddr,
    time::{Duration, Instant},
};
use tokio::timer::Delay;
use warp::{http::StatusCode, Filter};

#[macro_use]
extern crate diesel_migrations;

///////////////////////////////////////////////////////////////////////

const SERVER_LISTENING_DELAY: Duration = Duration::from_secs(1);

static INDEX_HTML: &str = include_str!("../../../resources/index.html");
static OPENAPI_YAML: &str = include_str!("../../../resources/openapi.yaml");

diesel_migrations::embed_migrations!("repo-sqlite/migrations");

fn create_connection_pool(url: &str, max_size: u32) -> Result<SqliteConnectionPool, Error> {
    log::info!("Creating SQLite connection pool for '{}'", url);
    let manager = SqliteConnectionManager::new(url);
    let pool = SqliteConnectionPool::builder()
        .max_size(max_size)
        .build(manager)?;
    Ok(pool)
}

fn initialize_database(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    log::info!("Initializing database");
    let connection = &*connection_pool.get()?;
    sql_query("PRAGMA automatic_index=0;").execute(connection)?;
    sql_query("PRAGMA foreign_keys=1;").execute(connection)?;
    sql_query("PRAGMA encoding='UTF-8';").execute(connection)?;
    Ok(())
}

fn migrate_database_schema(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    log::info!("Migrating database schema");
    let connection = &*connection_pool.get()?;
    embedded_migrations::run(connection)?;
    Ok(())
}

fn cleanup_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    log::info!("Cleaning up database storage");
    let connection = &*connection_pool.get()?;
    let helper = TrackRepositoryHelper::new(connection);
    connection.transaction::<_, Error, _>(|| helper.cleanup())
}

fn restore_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    log::info!("Restoring database storage");
    let collection_prototype = Collection {
        name: "Missing Collection".into(),
        description: Some("Recreated by aoide".into()),
    };
    let connection = &*connection_pool.get()?;
    let helper = TrackRepositoryHelper::new(connection);
    connection.transaction::<_, Error, _>(|| {
        helper.recreate_missing_collections(&collection_prototype)
    })?;
    Ok(())
}

fn optimize_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    log::info!("Optimizing database storage");
    let connection = &*connection_pool.get()?;
    sql_query("PRAGMA optimize;").execute(connection)?;
    Ok(())
}

pub fn main() -> Result<(), Error> {
    let started_at = chrono::Utc::now();

    let arg_matches = cli::ArgMatches::new(
        App::new(env!("CARGO_PKG_NAME"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION")),
    );

    env::init_logger(arg_matches.log_level_filter());

    if let Ok(exe_path) = current_exe() {
        log::info!("Executable: {}", exe_path.display());
    }
    log::info!("Version: {}", env!("CARGO_PKG_VERSION"));

    let database_url = arg_matches.database_url();
    log::info!("Database URL: {}", database_url);

    let listen_addr = arg_matches
        .listen_addr()
        .parse::<SocketAddr>()
        .map_err(|err| {
            log::error!("Invalid listen address: {}", arg_matches.listen_addr());
            err
        })?;
    log::info!("Network listen address: {}", listen_addr);

    // Workaround: Use a pool of size 1 to avoid 'database is locked'
    // errors due to multi-threading.
    let connection_pool = create_connection_pool(&database_url, 1)
        .expect("Failed to create database connection pool");

    initialize_database(&connection_pool).expect("Failed to initialize database");
    if arg_matches.skip_database_maintenance() {
        log::info!("Skipping database maintenance tasks");
    } else {
        migrate_database_schema(&connection_pool).expect("Failed to migrate database schema");
        cleanup_database_storage(&connection_pool).expect("Failed to cleanup database storage");
        restore_database_storage(&connection_pool).expect("Failed to restore database storage");
    }

    let sqlite_exec = SqliteExecutor::new(connection_pool.clone());

    log::info!("Creating service routes");

    let pooled_connection = warp::any()
        .map({ move || sqlite_exec.pooled_connection() })
        .and_then(|res: Result<_, _>| res.map_err(warp::reject::custom));

    // POST /shutdown
    let (server_shutdown_tx, server_shutdown_rx) = futures::sync::mpsc::unbounded::<()>();
    let shutdown_filter = warp::post2()
        .and(warp::path("shutdown"))
        .and(warp::path::end())
        .map(move || {
            server_shutdown_tx
                .unbounded_send(())
                .map(|()| StatusCode::ACCEPTED)
                .or_else(|_| {
                    log::warn!("Failed to forward shutdown request");
                    Ok(StatusCode::BAD_GATEWAY)
                })
        });

    // GET /about
    let about_filter = warp::get2()
        .and(warp::path("about"))
        .and(warp::path::end())
        .map(move || {
            warp::reply::json(&serde_json::json!({
            "name": env!("CARGO_PKG_NAME"),
            "description": env!("CARGO_PKG_DESCRIPTION"),
            "version": env!("CARGO_PKG_VERSION"),
            "authors": env!("CARGO_PKG_AUTHORS"),
            "instance": {
                "startedAt": started_at,
                }
            }))
        });

    // /collections
    let collections = warp::path("collections");
    let collections_uid = collections.and(warp::path::param::<aoide_core::entity::EntityUid>());
    let collections_create = warp::post2()
        .and(collections)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_create(body)
        });
    let collections_update = warp::put2()
        .and(collections_uid)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_update(query, body)
        });
    let collections_delete = warp::delete2()
        .and(collections_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_delete(uid)
        });
    let collections_list = warp::get2()
        .and(collections)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_list(query)
        });
    let collections_load = warp::get2()
        .and(collections_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|uid, query, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_load(uid, query)
        });
    let collections_resources = collections_list
        .or(collections_load)
        .or(collections_create)
        .or(collections_update)
        .or(collections_delete);

    // /tracks
    let tracks = warp::path("tracks");
    let tracks_uid = tracks.and(warp::path::param::<aoide_core::entity::EntityUid>());
    let tracks_create = warp::post2()
        .and(tracks)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_create(body)
        });
    let tracks_update = warp::put2()
        .and(tracks_uid)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|uid, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_update(uid, body)
        });
    let tracks_delete = warp::delete2()
        .and(tracks_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_delete(uid)
        });
    let tracks_load = warp::get2()
        .and(tracks_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| TracksHandler::new(pooled_connection).handle_load(uid));
    let tracks_list = warp::get2()
        .and(tracks)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_list(query)
        });
    let tracks_locate = warp::post2()
        .and(tracks)
        .and(warp::path("locate"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_locate(query, body)
        });
    let tracks_search = warp::post2()
        .and(tracks)
        .and(warp::path("search"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_search(query, body)
        });
    let tracks_replace = warp::post2()
        .and(tracks)
        .and(warp::path("replace"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_replace(query, body)
        });
    let tracks_purge = warp::post2()
        .and(tracks)
        .and(warp::path("purge"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, uri_predicates, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_purge(query, uri_predicates)
        });
    let tracks_relocate = warp::post2()
        .and(tracks)
        .and(warp::path("relocate"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |query, uri_relocations: Vec<UriRelocation>, pooled_connection| {
                TracksHandler::new(pooled_connection)
                    .handle_relocate(query, uri_relocations.into_iter())
            },
        );
    let tracks_resources = tracks_create
        .or(tracks_update)
        .or(tracks_delete)
        .or(tracks_load)
        .or(tracks_list)
        .or(tracks_locate)
        .or(tracks_search)
        .or(tracks_replace)
        .or(tracks_purge)
        .or(tracks_relocate);

    let albums_count_tracks = warp::post2()
        .and(warp::path("albums"))
        .and(warp::path("count-tracks"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_albums_count_tracks(query, body)
        });
    let albums_resources = albums_count_tracks;

    // /tags
    let tags_count_tracks = warp::post2()
        .and(warp::path("tags"))
        .and(warp::path("count-tracks"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_tags_count_tracks(query, body)
        });
    let tags_facets_count_tracks = warp::post2()
        .and(warp::path("tags"))
        .and(warp::path("facets"))
        .and(warp::path("count-tracks"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_tags_facets_count_tracks(query, body)
        });
    let tags_resources = tags_count_tracks.or(tags_facets_count_tracks);

    // Static content
    let index_html = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let openapi_yaml = warp::path("openapi.yaml").map(|| {
        warp::reply::with_header(
            OPENAPI_YAML,
            "Content-Type",
            "application/x-yaml;charset=utf-8",
        )
    });
    let static_resources = index_html.or(openapi_yaml);

    log::info!("Initializing server");
    let server = warp::serve(
        collections_resources
            .or(tracks_resources)
            .or(albums_resources)
            .or(tags_resources)
            .or(static_resources)
            .or(shutdown_filter)
            .or(about_filter),
    );
    let (socket_addr, server_listener) = server.bind_with_graceful_shutdown(
        listen_addr,
        server_shutdown_rx.into_future().map(|_| {
            log::info!("Shutting down server");
        }),
    );
    log::info!("Starting");
    let main_task = future::lazy(move || {
        // Give the server some time for starting up before announcing the
        // actual endpoint address, i.e. when using an ephemeral port.
        Delay::new(Instant::now() + SERVER_LISTENING_DELAY)
            .map(move |()| {
                // stderr
                log::info!("Listening on {}", socket_addr);
                // stdout
                println!("{}", socket_addr);
            })
            .map_err(drop)
            .join(
                server_listener
                    .map(drop)
                    .map_err(drop)
                    .map(|()| log::info!("Finished"))
                    .map_err(|()| log::error!("Aborted")),
            )
            .map(drop)
    });
    tokio::run(main_task);
    log::info!("Stopped");

    optimize_database_storage(&connection_pool).expect("Failed to optimize database storage");

    log::info!("Exiting");
    Ok(())
}
