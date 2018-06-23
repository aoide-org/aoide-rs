// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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
use aoide::api::web::*;

use aoide_core::domain::collection::Collection;

use aoide_storage::storage::track::util::TrackRepositoryHelper;

use actix::prelude::*;
use actix_web::*;

use clap::App;

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use failure::Error;

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use std::env;

embed_migrations!("aoide-storage/db/migrations/sqlite");

fn create_connection_pool(url: &str, max_size: u32) -> Result<SqliteConnectionPool, Error> {
    info!("Creating SQLite connection pool for '{}'", url);
    let manager = ConnectionManager::new(url);
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

fn repair_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    info!("Repairing database storage");
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
        info!("Skipping database maintenance");
    } else {
        migrate_database_schema(&connection_pool).unwrap();
        cleanup_database_storage(&connection_pool).unwrap();
        repair_database_storage(&connection_pool).unwrap();
    }

    info!("Creating actor system");
    let sys = actix::System::new(env!("CARGO_PKG_NAME"));

    info!("Registering route handlers");
    let addr = SyncArbiter::start(3, move || SqliteExecutor::new(connection_pool.clone()));
    server::new(move || {
        actix_web::App::with_state(AppState {
                executor: addr.clone(),
            })
            .middleware(actix_web::middleware::Logger::default()) // enable logger
            .prefix("/")
            .resource("/tracks", |r| {
                r.method(http::Method::GET).with_async(on_list_tracks);
                r.method(http::Method::POST).with_async(on_create_track);
            })
            .resource("/tracks/search", |r| {
                r.method(http::Method::POST).with_async(on_search_tracks);
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
                r.method(http::Method::POST).with_async(on_replace_tracks);
            })
            .resource("/tracks/locate", |r| {
                r.method(http::Method::POST).with_async(on_locate_tracks);
            })
            .resource("/tracks/{uid}", |r| {
                r.method(http::Method::GET).with_async(on_load_track);
                r.method(http::Method::PUT).with_async(on_update_track);
                r.method(http::Method::DELETE).with_async(on_delete_track);
            })
            .resource("/collections", |r| {
                r.method(http::Method::GET).with_async(on_list_collections);
                r.method(http::Method::POST).with_async(on_create_collection);
            })
            .resource("/collections/{uid}", |r| {
                r.method(http::Method::GET).with_async(on_load_collection);
                r.method(http::Method::PUT).with_async(on_update_collection);
                r.method(http::Method::DELETE).with_async(on_delete_collection);
            })
            .default_resource(|r| {
                r.method(http::Method::GET).f(|_req| HttpResponse::NotFound());
                r.route().filter(pred::Not(pred::Get())).f(
                    |_req| HttpResponse::MethodNotAllowed());
            })
    }).bind(listen_addr)
        .unwrap()
        .start();

    info!("Running actor system");
    let _ = sys.run();
    info!("Stopped actor system");

    info!("Exiting");
    Ok(())
}
