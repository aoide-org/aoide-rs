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

extern crate futures;

extern crate gotham;
#[macro_use]
extern crate gotham_derive;
extern crate hyper;
extern crate mime;
extern crate serde;

extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
extern crate gotham_middleware_diesel;
extern crate r2d2;
extern crate r2d2_diesel;

extern crate env_logger;

#[macro_use]
extern crate log;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

use diesel::prelude::*;

use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

use futures::{future, Future, Stream};
// futures v0.2.1
//use futures::{future, Future};
//use futures::stream::{Stream, StreamExt};

use gotham::helpers::http::response::create_response;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::state::{FromState, State};
use gotham::handler::{HandlerFuture, IntoHandlerError};

use gotham_middleware_diesel::DieselMiddleware;

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use std::env;

use aoide::domain::collection::*;
use aoide::storage::collection::*;
use aoide::usecases::{Collections};

embed_migrations!("db/migrations/sqlite");

type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
type PooledSqliteConnection = PooledConnection<ConnectionManager<SqliteConnection>>;
type SqliteDieselMiddleware = DieselMiddleware<SqliteConnection>;

fn create_connection_pool(url: &str) -> Result<SqliteConnectionPool, r2d2::Error> {
    info!("Creating SQLite connection pool for '{}'", url);
    let manager = ConnectionManager::new(url);
    SqliteConnectionPool::new(manager)
}

#[derive(Debug)]
struct MigrationError;

impl From<r2d2::Error> for MigrationError {
    fn from(_from: r2d2::Error) -> Self {
        MigrationError {}
    }
}

impl From<diesel_migrations::RunMigrationsError> for MigrationError {
    fn from(_from: diesel_migrations::RunMigrationsError) -> Self {
        MigrationError {}
    }
}

fn migrate_database_schema(connection_pool: &SqliteConnectionPool) -> Result<(), MigrationError> {
    info!("Migrating database schema");
    let pooled_connection = connection_pool.get()?;
    embedded_migrations::run(&*pooled_connection)?;
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

fn init_env_logger_verbosity(verbosity: u8) {
    let log_level_filter = match verbosity {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace,
    };
    init_env_logger(log_level_filter);
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct QueryStringExtractor {
    name: String,
}

fn post_collection_handler(mut state: State) -> (State, hyper::Response) {
    let res = {
        let query_param = QueryStringExtractor::take_from(&mut state);
        let collection = CollectionEntity::with_name(query_param.name);
        info!("Created new collection: {}", collection.header().uid());
        if log_enabled!(log::Level::Debug) {
            debug!("Responding with: {:#?}", collection);
        }
        create_response(
            &state,
            hyper::StatusCode::Created,
            Some((
                serde_json::to_vec(&collection).expect("serialized collection"),
                mime::APPLICATION_JSON,
            )),
        )
    };
    (state, res)
}

fn async_post_collection_handler(mut state: State) -> Box<HandlerFuture> {
    let f = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                info!("Parsing request body");
                let mut collection_body: CollectionBody = match serde_json::from_slice(&valid_body)
                {
                    Ok(p) => p,
                    Err(e) => {
                        return future::err((
                            state,
                            e.into_handler_error()
                                .with_status(hyper::StatusCode::BadRequest),
                        ))
                    }
                };
                info!("Obtaining pooled connection");
                let connection: PooledSqliteConnection =
                    gotham_middleware_diesel::state_data::connection(&state);
                info!("Instantiating repository");
                let repository = CollectionRepository::new(&*connection);
                info!("Creating entity");
                let collection = match repository.create_entity(collection_body.name) {
                    Ok(collection) => collection,
                    Err(e) => return future::err((state, e.into_handler_error())),
                };
                info!("Preparing response");
                let response = match serde_json::to_vec(&collection) {
                    Ok(response_body) => create_response(
                        &state,
                        hyper::StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => return future::err((state, e.into_handler_error())),
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(f)
}

fn router(middleware: SqliteDieselMiddleware) -> Router {
    // Create a new pipeline set
    let editable_pipeline_set = new_pipeline_set();

    // Add the middleware to a new pipeline
    let (editable_pipeline_set, pipeline) =
        editable_pipeline_set.add(new_pipeline().add(middleware).build());
    let pipeline_set = finalize_pipeline_set(editable_pipeline_set);

    let default_pipeline_chain = (pipeline, ());

    // Build the router
    build_router(default_pipeline_chain, pipeline_set, |route| {
        route
            .post("/collections")
            //.with_query_string_extractor::<QueryStringExtractor>()
            .to(async_post_collection_handler);
    })
}

pub fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        println!("usage: {} <DB_URL>", args[0]);
        return;
    }

    // TODO: Parse verbosity from args
    init_env_logger_verbosity(2);

    let db_url = match args.len() {
        2 => &args[1],
        _ => ":memory:",
    };

    let connection_pool = create_connection_pool(db_url).unwrap();

    migrate_database_schema(&connection_pool).unwrap();

    info!("Creating middleware");
    let middleware = DieselMiddleware::with_pool(connection_pool);

    let router = router(middleware);

    let listen_addr = "127.0.0.1:7878";
    info!("Listening for requests at http://{}", listen_addr);
    gotham::start(listen_addr, router)
}
