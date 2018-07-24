// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

extern crate aoide;

extern crate aoide_core;

extern crate aoide_storage;

extern crate actix;

extern crate actix_web;

extern crate clap;

extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

extern crate env_logger;

extern crate failure;

#[macro_use]
extern crate log;

use aoide::api::cli::*;
use aoide::api::web::{albums::*, collections::*, tracks::*, *};

use aoide_core::domain::collection::Collection;

use aoide_storage::storage::track::util::TrackRepositoryHelper;

use actix::prelude::*;
use actix_web::{error, fs, http, pred, server, HttpResponse};

use clap::App;

use diesel::prelude::*;

use failure::Error;

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use std::env;

embed_migrations!("aoide-storage/resources/migrations/sqlite");

fn create_connection_pool(url: &str, max_size: u32) -> Result<SqliteConnectionPool, Error> {
    info!("Creating SQLite connection pool for '{}'", url);
    let manager = SqliteConnectionManager::new(url);
    let pool = SqliteConnectionPool::builder()
        .max_size(max_size)
        .build(manager)?;
    Ok(pool)
}

fn migrate_database_schema(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    info!("Migrating database schema");
    let connection = &*connection_pool.get()?;
    embedded_migrations::run(connection)?;
    Ok(())
}

fn cleanup_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    info!("Cleaning up database storage");
    let connection = &*connection_pool.get()?;
    let helper = TrackRepositoryHelper::new(connection);
    connection.transaction::<_, Error, _>(|| helper.cleanup())
}

fn restore_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    info!("Restoring database storage");
    let collection_prototype = Collection {
        name: "Missing Collection".into(),
        description: Some("Recreated by aoide".into()),
    };
    let connection = &*connection_pool.get()?;
    let helper = TrackRepositoryHelper::new(connection);
    connection
        .transaction::<_, Error, _>(|| helper.recreate_missing_collections(&collection_prototype))?;
    Ok(())
}

fn init_env_logger(log_level_filter: LogLevelFilter) {
    let mut logger_builder = LoggerBuilder::new();

    println!("Setting log level filter to {}", log_level_filter);
    logger_builder.filter(None, log_level_filter);

    if env::var("RUST_LOG").is_ok() {
        let rust_log_var = &env::var("RUST_LOG").unwrap();
        println!("Parsing RUST_LOG={}", rust_log_var);
        logger_builder.parse(rust_log_var);
    }

    logger_builder.init();
}

fn web_app(executor: &Addr<SqliteExecutor>) -> actix_web::App<AppState> {
    actix_web::App::with_state(AppState {
                executor: executor.clone(),
            })
            .middleware(actix_web::middleware::Logger::default()) // enable logger
            .prefix("/")
            .resource("/tracks", |r| {
                r.method(http::Method::GET).with_async(on_list_tracks);
                r.method(http::Method::POST).with_async_config(on_create_track,
                    |(_, cfg_body)| { cfg_body.error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
            })
            .resource("/tracks/search", |r| {
                r.method(http::Method::POST).with_async_config(on_search_tracks,
                    |(_, _, _, cfg_body)| { cfg_body.error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
            })
            .resource("/tracks/fields", |r| {
                r.method(http::Method::GET).with_async(on_list_tracks_fields);
            })
            .resource("/tracks/tags", |r| {
                r.method(http::Method::GET).with_async(on_list_tracks_tags);
            })
            .resource("/tracks/tags/facets", |r| {
                r.method(http::Method::GET).with_async(on_list_tracks_tags_facets);
            })
            .resource("/tracks/replace", |r| {
                r.method(http::Method::POST).with_async_config(on_replace_tracks,
                    // Limit maximum body size to 1 MB (Default: 256 KB)
                    |(_, _, cfg_body)| { cfg_body.limit(1024 * 1024).error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
            })
            .resource("/tracks/locate", |r| {
                r.method(http::Method::POST).with_async_config(on_locate_tracks,
                    |(_, _, _, cfg_body)| { cfg_body.error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
            })
            .resource("/tracks/{uid}", |r| {
                r.method(http::Method::GET).with_async(on_load_track);
                r.method(http::Method::PUT).with_async_config(on_update_track,
                    |(_, _, cfg_body)| { cfg_body.error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
                r.method(http::Method::DELETE).with_async(on_delete_track);
            })
            .resource("/albums", |r| {
                r.method(http::Method::GET).with_async(on_list_albums);
            })
            .resource("/collections", |r| {
                r.method(http::Method::GET).with_async(on_list_collections);
                r.method(http::Method::POST).with_async_config(on_create_collection,
                    |(_, cfg_body)| { cfg_body.error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
            })
            .resource("/collections/{uid}", |r| {
                r.method(http::Method::GET).with_async(on_load_collection);
                r.method(http::Method::PUT).with_async_config(on_update_collection,
                    |(_, _, cfg_body)| { cfg_body.error_handler(|err, _req| {
                        let err_msg = format!("{}", err);
                        error::InternalError::from_response(
                            err, HttpResponse::BadRequest().body(err_msg)).into()
                        });
                    });
                r.method(http::Method::DELETE).with_async(on_delete_collection);
            })
            .handler("/", fs::StaticFiles::new("./resources/").expect("Missing resources folder").index_file("index.html"))
            .default_resource(|r| {
                r.method(http::Method::GET).f(|_req| HttpResponse::NotFound());
                r.route().filter(pred::Not(pred::Get())).f(
                    |_req| HttpResponse::MethodNotAllowed());
            })
}

pub fn main() -> Result<(), Error> {
    let arg_matches = ArgMatches::new(
        App::new(env!("CARGO_PKG_NAME"))
            .author(env!("CARGO_PKG_AUTHORS"))
            .version(env!("CARGO_PKG_VERSION"))
            .about(env!("CARGO_PKG_DESCRIPTION")),
    );

    init_env_logger(arg_matches.log_level_filter());

    let database_url = arg_matches.database_url();
    info!("Database URL: {}", database_url);

    let listen_addr = arg_matches.listen_addr();
    info!("Network listen address: {}", listen_addr);

    // Workaround: Use a pool of size 1 to avoid 'database is locked'
    // errors due to multi-threading.
    let connection_pool =
        create_connection_pool(database_url, 1).expect("Failed to create database connection pool");

    if arg_matches.skip_database_maintenance() {
        info!("Skipping database maintenance tasks");
    } else {
        migrate_database_schema(&connection_pool).expect("Failed to migrate database schema");
        cleanup_database_storage(&connection_pool).expect("Failed to cleanup database storage");
        restore_database_storage(&connection_pool).expect("Failed to restore database storage");
    }

    let sys_name = env!("CARGO_PKG_NAME");
    info!("Creating actor system '{}'", sys_name);
    let sys = actix::System::new(env!("CARGO_PKG_NAME"));

    let num_worker_threads = 3;
    info!("Starting {} executor worker threads", num_worker_threads);
    let executor = SyncArbiter::start(num_worker_threads, move || {
        SqliteExecutor::new(connection_pool.clone())
    });

    info!("Registering route handlers");
    server::HttpServer::new(move || web_app(&executor))
        .bind(listen_addr)
        .unwrap_or_else(|_| panic!("Failed to bind listen address '{}'", listen_addr))
        .start();

    info!("Running actor system");
    let _ = sys.run();
    info!("Stopped actor system");

    info!("Exiting");
    Ok(())
}
