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

use aoide::api::web::{collections::*, tracks::*, *};
use aoide_core::collection::Collection;
use aoide_storage::{api::UriPredicate, storage::track::util::TrackRepositoryHelper};

use clap::App;
use diesel::{prelude::*, sql_query};
use failure::Error;
use futures::{future, Future, Stream};
use std::net::SocketAddr;
use warp::{http::StatusCode, Filter};

#[macro_use]
extern crate diesel_migrations;

///////////////////////////////////////////////////////////////////////

static INDEX_HTML: &str = include_str!("../../../resources/index.html");
static OPENAPI_YAML: &str = include_str!("../../../resources/openapi.yaml");

diesel_migrations::embed_migrations!("storage/resources/migrations/sqlite");

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
    let connection_pool =
        create_connection_pool(database_url, 1).expect("Failed to create database connection pool");

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
        .and(collections.and(warp::path::end()))
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_create(body)
        });
    let collections_update = warp::put2()
        .and(collections_uid.and(warp::path::end()))
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_update(query, body)
        });
    let collections_delete = warp::delete2()
        .and(collections_uid.and(warp::path::end()))
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_delete(uid)
        });
    let collections_list = warp::get2()
        .and(collections.and(warp::path::end()))
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query, pooled_connection| {
            CollectionsHandler::new(pooled_connection).handle_list(query)
        });
    let collections_load = warp::get2()
        .and(collections_uid.and(warp::path::end()))
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
        .and(tracks.and(warp::path::end()))
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_create(body)
        });
    let tracks_update = warp::put2()
        .and(tracks_uid.and(warp::path::end()))
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|uid, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_update(uid, body)
        });
    let tracks_delete = warp::delete2()
        .and(tracks_uid.and(warp::path::end()))
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_delete(uid)
        });
    let tracks_load = warp::get2()
        .and(tracks_uid.and(warp::path::end()))
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| TracksHandler::new(pooled_connection).handle_load(uid));
    let tracks_list = warp::get2()
        .and(tracks.and(warp::path::end()))
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_list(query)
        });
    let tracks_search = warp::post2()
        .and(tracks.and(warp::path("search")).and(warp::path::end()))
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_search(query, body)
        });
    let tracks_locate = warp::post2()
        .and(tracks.and(warp::path("locate")).and(warp::path::end()))
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_locate(query, body)
        });
    let tracks_replace = warp::post2()
        .and(tracks.and(warp::path("replace")).and(warp::path::end()))
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_replace(query, body)
        });
    let tracks_purge = warp::post2()
        .and(tracks.and(warp::path("purge")).and(warp::path::end()))
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, uri_predicates: Vec<UriPredicate>, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_purge(query, uri_predicates.into_iter())
        });
    let tracks_albums_count = warp::post2()
        .and(
            tracks
                .and(warp::path("albums"))
                .and(warp::path("count"))
                .and(warp::path::end()),
        )
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_albums_count(query, body)
        });
    let tracks_tags_count = warp::post2()
        .and(
            tracks
                .and(warp::path("tags"))
                .and(warp::path("count"))
                .and(warp::path::end()),
        )
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_tags_count(query, body)
        });
    let tracks_tags_facets_count = warp::post2()
        .and(
            tracks
                .and(warp::path("tags"))
                .and(warp::path("facets"))
                .and(warp::path("count"))
                .and(warp::path::end()),
        )
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            TracksHandler::new(pooled_connection).handle_tags_facets_count(query, body)
        });
    let tracks_resources = tracks_search
        .or(tracks_replace)
        .or(tracks_purge)
        .or(tracks_list)
        .or(tracks_locate)
        .or(tracks_create)
        .or(tracks_update)
        .or(tracks_delete)
        .or(tracks_load)
        .or(tracks_albums_count)
        .or(tracks_tags_count)
        .or(tracks_tags_facets_count);

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
        tracks_resources
            .or(collections_resources)
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
    // Write the actual socket address (might use an ephemeral port)
    println!("{}", socket_addr);

    log::info!("Starting");
    let main_task = future::lazy(move || {
        log::info!("Running...");
        server_listener.map(drop).map_err(drop).then(|res| {
            match res {
                Ok(()) => log::info!("Finished"),
                Err(()) => log::error!("Aborted"),
            };
            res
        })
    });
    tokio::run(main_task);
    log::info!("Stopped");

    optimize_database_storage(&connection_pool).expect("Failed to optimize database storage");

    log::info!("Exiting");
    Ok(())
}
