// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod cli;
mod env;

use aoide::{
    api::web::{collections::*, playlists::*, reject_from_anyhow, tracks::*},
    *,
};

use aoide_core::{collection::Collection, entity::EntityUid};

mod _serde {
    pub use aoide_core_serde::entity::EntityUid;
}

use aoide_repo_sqlite::{
    playlist::util::RepositoryHelper as PlaylistRepositoryHelper,
    track::util::RepositoryHelper as TrackRepositoryHelper,
};

use anyhow::Error;
use clap::App;
use diesel::{prelude::*, sql_query};
use futures::future::{join, FutureExt};
use std::{env::current_exe, net::SocketAddr, time::Duration};
use tokio::{sync::mpsc, time::delay_for};
use warp::{http::StatusCode, Filter};

#[macro_use]
extern crate diesel_migrations;

///////////////////////////////////////////////////////////////////////

const WEB_SERVER_LISTENING_DELAY: Duration = Duration::from_millis(100);

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
    {
        let helper = TrackRepositoryHelper::new(connection);
        connection.transaction::<_, Error, _>(|| helper.cleanup())?;
    }
    {
        let helper = PlaylistRepositoryHelper::new(connection);
        connection.transaction::<_, Error, _>(|| helper.cleanup())?;
    }
    Ok(())
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

#[tokio::main]
pub async fn main() -> Result<(), Error> {
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
        .and_then(|res: Result<_, _>| async { res.map_err(reject_from_anyhow) });

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
            "authors": env!("CARGO_PKG_AUTHORS"),
            "instance": {
                "startedAt": started_at,
                }
            }))
        });

    // /collections
    let collections = warp::path("collections");
    let collections_uid = collections.and(warp::path::param::<EntityUid>());
    let collections_create = warp::post()
        .and(collections)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            async { CollectionsHandler::new(pooled_connection).handle_create(body) }
        });
    let collections_update = warp::put()
        .and(collections_uid)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { CollectionsHandler::new(pooled_connection).handle_update(query, body) }
        });
    let collections_delete = warp::delete()
        .and(collections_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            async { CollectionsHandler::new(pooled_connection).handle_delete(uid) }
        });
    let collections_list = warp::get()
        .and(collections)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query, pooled_connection| {
            async { CollectionsHandler::new(pooled_connection).handle_list(query) }
        });
    let collections_load = warp::get()
        .and(collections_uid)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|uid, query, pooled_connection| {
            async { CollectionsHandler::new(pooled_connection).handle_load(uid, query) }
        });
    let collections_filters = collections_list
        .or(collections_load)
        .or(collections_create)
        .or(collections_update)
        .or(collections_delete);

    // /playlists
    let playlists = warp::path("playlists");
    let playlists_uid = playlists.and(warp::path::param::<EntityUid>());
    let playlists_create = warp::post()
        .and(playlists)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            async { PlaylistsHandler::new(pooled_connection).handle_create(body) }
        });
    let playlists_update = warp::put()
        .and(playlists_uid)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { PlaylistsHandler::new(pooled_connection).handle_update(query, body) }
        });
    let playlists_patch = warp::patch()
        .and(playlists_uid)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { PlaylistsHandler::new(pooled_connection).handle_patch(query, body) }
        });
    let playlists_delete = warp::delete()
        .and(playlists_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            async { PlaylistsHandler::new(pooled_connection).handle_delete(uid) }
        });
    let playlists_list = warp::get()
        .and(playlists)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(move |query, pooled_connection| {
            async move { PlaylistsHandler::new(pooled_connection).handle_list(query) }
        });
    let playlists_load = warp::get()
        .and(playlists_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            async { PlaylistsHandler::new(pooled_connection).handle_load(uid) }
        });
    let playlists_filters = playlists_create
        .or(playlists_update)
        .or(playlists_patch)
        .or(playlists_delete)
        .or(playlists_load)
        .or(playlists_list);

    // /tracks
    let tracks = warp::path("tracks");
    let tracks_uid = tracks.and(warp::path::param::<EntityUid>());
    let tracks_create = warp::post()
        .and(tracks)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_create(body) }
        });
    let tracks_update = warp::put()
        .and(tracks_uid)
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|uid, body, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_update(uid, body) }
        });
    let tracks_delete = warp::delete()
        .and(tracks_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_delete(uid) }
        });
    let tracks_load = warp::get()
        .and(tracks_uid)
        .and(warp::path::end())
        .and(pooled_connection.clone())
        .and_then(|uid, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_load(uid) }
        });
    let tracks_load_batch = warp::post()
        .and(tracks)
        .and(warp::path("load"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|body: Vec<_serde::EntityUid>, pooled_connection| {
            async {
                TracksHandler::new(pooled_connection)
                    .handle_load_batch(body.into_iter().map(Into::into))
            }
        });
    let tracks_list = warp::get()
        .and(tracks)
        .and(warp::path::end())
        .and(warp::query())
        .and(pooled_connection.clone())
        .and_then(|query, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_list(query) }
        });
    let tracks_locate = warp::post()
        .and(tracks)
        .and(warp::path("locate"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(move |query, body, pooled_connection| {
            async move { TracksHandler::new(pooled_connection).handle_locate(query, body) }
        });
    let tracks_resolve = warp::post()
        .and(tracks)
        .and(warp::path("resolve"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(move |query, body, pooled_connection| {
            async move { TracksHandler::new(pooled_connection).handle_resolve(query, body) }
        });
    let tracks_search = warp::post()
        .and(tracks)
        .and(warp::path("search"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_search(query, body) }
        });
    let tracks_replace = warp::post()
        .and(tracks)
        .and(warp::path("replace"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_replace(query, body) }
        });
    let tracks_purge = warp::post()
        .and(tracks)
        .and(warp::path("purge"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, uri_predicates, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_purge(query, uri_predicates) }
        });
    let tracks_relocate = warp::post()
        .and(tracks)
        .and(warp::path("relocate"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(
            |query, uri_relocations: Vec<UriRelocation>, pooled_connection| {
                async {
                    TracksHandler::new(pooled_connection)
                        .handle_relocate(query, uri_relocations.into_iter())
                }
            },
        );
    let tracks_filters = tracks_create
        .or(tracks_update)
        .or(tracks_delete)
        .or(tracks_load)
        .or(tracks_load_batch)
        .or(tracks_list)
        .or(tracks_locate)
        .or(tracks_resolve)
        .or(tracks_search)
        .or(tracks_replace)
        .or(tracks_purge)
        .or(tracks_relocate);

    let albums_count_tracks = warp::post()
        .and(warp::path("albums"))
        .and(warp::path("count-tracks"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_albums_count_tracks(query, body) }
        });
    let albums_filters = albums_count_tracks;

    // /tags
    let tags_count_tracks = warp::post()
        .and(warp::path("tags"))
        .and(warp::path("count-tracks"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection.clone())
        .and_then(|query, body, pooled_connection| {
            async { TracksHandler::new(pooled_connection).handle_tags_count_tracks(query, body) }
        });
    let tags_facets_count_tracks = warp::post()
        .and(warp::path("tags"))
        .and(warp::path("facets"))
        .and(warp::path("count-tracks"))
        .and(warp::path::end())
        .and(warp::query())
        .and(warp::body::json())
        .and(pooled_connection)
        .and_then(|query, body, pooled_connection| {
            async {
                TracksHandler::new(pooled_connection).handle_tags_facets_count_tracks(query, body)
            }
        });
    let tags_filters = tags_count_tracks.or(tags_facets_count_tracks);

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

    let cors = warp::cors().allow_any_origin();
    let server = warp::serve(
        collections_filters
            .or(playlists_filters)
            .or(tracks_filters)
            .or(albums_filters)
            .or(tags_filters)
            .or(static_filters)
            .or(shutdown_filter)
            .or(about_filter)
            .with(cors),
    );

    log::info!("Starting");

    let (socket_addr, server_listener) =
        server.bind_with_graceful_shutdown(listen_addr, async move {
            server_shutdown_rx.recv().await;
            log::info!("Shutting down");
        });

    let server_listening = async move {
        // Give the server some time to become ready and start listening
        // before announcing the actual endpoint address, i.e. when using
        // an ephemeral port. The delay might need to be tuned depending
        // on how long the startup actually takes. Unfortunately warp does
        // not provide any signal when the server has started listening.
        delay_for(WEB_SERVER_LISTENING_DELAY).await;

        // -> stderr
        log::info!("Listening on {}", socket_addr);
        // -> stdout
        println!("{}", socket_addr);
    };

    join(server_listener, server_listening).map(drop).await;

    log::info!("Stopped");

    optimize_database_storage(&connection_pool).expect("Failed to optimize database storage");

    log::info!("Exiting");
    Ok(())
}
