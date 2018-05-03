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

extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

extern crate env_logger;

#[macro_use]
extern crate failure;

extern crate futures;

extern crate gotham;

#[macro_use]
extern crate gotham_derive;

extern crate gotham_middleware_diesel;

extern crate hyper;

#[macro_use]
extern crate log;

extern crate mime;

extern crate r2d2;

extern crate r2d2_diesel;

extern crate rmp_serde;

extern crate serde;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

use aoide_core::domain::collection::*;
use aoide_core::domain::track::*;
use aoide_core::domain::entity::*;
use aoide::storage::collection::*;
use aoide::storage::track::*;
use aoide::storage::SerializedEntity;
use aoide::usecases::*;

use diesel::prelude::*;

use futures::{future, Future, Stream};
use futures::future::IntoFuture;
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

use hyper::{Response, StatusCode};
use hyper::header::{ContentType, Headers};

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use r2d2::{Pool, PooledConnection};
use r2d2_diesel::ConnectionManager;

use std::env;

embed_migrations!("db/migrations/sqlite");

type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
type PooledSqliteConnection = PooledConnection<ConnectionManager<SqliteConnection>>;
type SqliteDieselMiddleware = DieselMiddleware<SqliteConnection>;

fn create_connection_pool(url: &str) -> Result<SqliteConnectionPool, failure::Error> {
    info!("Creating SQLite connection pool for '{}'", url);
    let manager = ConnectionManager::new(url);
    let pool = SqliteConnectionPool::new(manager)?;
    Ok(pool)
}

fn migrate_database_schema(connection_pool: &SqliteConnectionPool) -> Result<(), failure::Error> {
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

fn init_env_logger_verbosity(verbosity_level: u8) {
    let log_level_filter = match verbosity_level {
        0 => LogLevelFilter::Error,
        1 => LogLevelFilter::Warn,
        2 => LogLevelFilter::Info,
        3 => LogLevelFilter::Debug,
        _ => LogLevelFilter::Trace,
    };
    init_env_logger(log_level_filter);
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct UidPathExtractor {
    uid: String,
}

fn find_collection(connection: &SqliteConnection, uid: &EntityUid) -> Result<Option<CollectionEntity>, failure::Error> {
    let repository = CollectionRepository::new(connection);
    let result = repository.find_entity(&uid)?;
    Ok(result)
}

fn handle_get_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let path = UidPathExtractor::take_from(&mut state);
    let uid: EntityUid = path.uid.into();

    let connection = &*gotham_middleware_diesel::state_data::connection(&state);

    let result = match find_collection(connection, &uid) {
        Ok(Some(collection)) => match serde_json::to_vec(&collection) {
            Ok(response_body) => {
                let response = create_response(
                    &state,
                    StatusCode::Ok,
                    Some((response_body, mime::APPLICATION_JSON)),
                );
                Ok((state, response))
            }
            Err(e) => Err((state, e.into_handler_error())),
        },
        Ok(None) => {
            let response = create_response(&state, StatusCode::NotFound, None);
            Ok((state, response))
        }
        Err(e) => Err((state, e.compat().into_handler_error())),
    };

    Box::new(result.into_future())
}

fn remove_collection(connection: &SqliteConnection, uid: &EntityUid) -> Result<Option<()>, failure::Error> {
    let repository = CollectionRepository::new(connection);
    let result = repository.remove_entity(&uid)?;
    Ok(result)
}

fn handle_delete_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let path = UidPathExtractor::take_from(&mut state);
    let uid: EntityUid = path.uid.into();

    let connection = &*gotham_middleware_diesel::state_data::connection(&state);

    let result = match remove_collection(connection, &uid) {
        Ok(Some(_)) => {
            let response = create_response(&state, StatusCode::Ok, None);
            future::ok((state, response))
        }
        Ok(None) => {
            let response = create_response(&state, StatusCode::Accepted, None);
            future::ok((state, response))
        }
        Err(e) => future::err((state, e.compat().into_handler_error())),
    };

    Box::new(result.into_future())
}

#[derive(Debug, Deserialize, StateData, StaticResponseExtender)]
struct PaginationQueryStringExtractor {
    offset: Option<PaginationOffset>,
    limit: Option<PaginationLimit>,
}

fn find_all_collections(connection: &SqliteConnection, pagination: &Pagination) -> Result<Vec<CollectionEntity>, failure::Error> {
    let repository = CollectionRepository::new(connection);
    let result = repository.find_all_entities(pagination)?;
    Ok(result)
}

fn handle_get_collections_path_pagination(mut state: State) -> Box<HandlerFuture> {
    let query_params = PaginationQueryStringExtractor::take_from(&mut state);
    let pagination = Pagination {
        offset: query_params.offset,
        limit: query_params.limit,
    };

    let connection = &*gotham_middleware_diesel::state_data::connection(&state);

    let handler_future = match find_all_collections(connection, &pagination) {
        Ok(collections) => match serde_json::to_vec(&collections) {
            Ok(response_body) => {
                let response = create_response(
                    &state,
                    StatusCode::Ok,
                    Some((response_body, mime::APPLICATION_JSON)),
                );
                future::ok((state, response))
            }
            Err(e) => future::err((state, e.into_handler_error())),
        },
        Err(e) => future::err((state, e.compat().into_handler_error())),
    };

    Box::new(handler_future)
}

fn create_collection(connection: &SqliteConnection, body: CollectionBody) -> Result<CollectionEntity, failure::Error> {
    let repository = CollectionRepository::new(connection);
    let result = repository.create_entity(body)?;
    Ok(result)
}

fn handle_post_collections(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity_body: CollectionBody = match serde_json::from_slice(&valid_body)
                {
                    Ok(p) => p,
                    Err(e) => {
                        return future::err((
                            state,
                            e.into_handler_error().with_status(StatusCode::BadRequest),
                        ))
                    }
                };

                let connection = &*gotham_middleware_diesel::state_data::connection(&state);

                let entity = match create_collection(connection, entity_body) {
                    Ok(entity) => entity,
                    Err(e) => {
                        return future::err((
                            state,
                            e.compat().into_handler_error(),
                        ))
                    }
                };

                let response = match serde_json::to_vec(&entity.header()) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => return future::err((state, e.into_handler_error())),
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(handler_future)
}

fn update_collection(connection: &SqliteConnection, entity: &CollectionEntity) -> Result<Option<EntityRevision>, failure::Error> {
    let repository = CollectionRepository::new(connection);
    let result = repository.update_entity(entity)?;
    Ok(result)
}

fn handle_put_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity: CollectionEntity = match serde_json::from_slice(&valid_body)
                {
                    Ok(p) => p,
                    Err(e) => {
                        warn!("Failed to deserialize request body - {}", e);
                        return future::err((
                            state,
                            e.into_handler_error().with_status(StatusCode::BadRequest),
                        ))
                    }
                };

                let path = UidPathExtractor::take_from(&mut state);
                let uid = EntityUid::from(path.uid);
                let entity_uid = entity.header().uid();
                if &uid != entity_uid {
                    let e = format_err!("Mismatching identifiers: expected = {}, actual = {}", uid, entity_uid);
                    warn!("Failed to validate request - {}", e);
                    return future::err((
                        state,
                        e.compat().into_handler_error().with_status(StatusCode::BadRequest),
                    ))
                }

                let connection = &*gotham_middleware_diesel::state_data::connection(&state);

                let next_revision = match update_collection(connection, &entity) {
                    Ok(Some(next_revision)) => next_revision,
                    Ok(None) => {
                        let e = format_err!("Unknown or revision mismatch: {:?}", entity.header());
                        warn!("Failed to update collection - {}", e);
                        return future::err((state, failure::Error::from(e).compat().into_handler_error().with_status(StatusCode::NotFound)))
                    }
                    Err(e) => {
                        warn!("Failed to update collection - {}", e);
                        return future::err((state, e.compat().into_handler_error()))
                    }
                };

                let response = match serde_json::to_vec(&EntityHeader::new(uid, next_revision)) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Ok,
                        Some((response_body, mime::APPLICATION_JSON))
                    ),
                    Err(e) => {
                        warn!("Failed to serialize response body - {}", e);
                        return future::err((state, e.into_handler_error()))
                    }
                };
                future::ok((state, response))
            }
            Err(e) => {
                warn!("Failed to read request body - {}", e);
                future::err((state, e.into_handler_error()))
            }
        });

    Box::new(handler_future)
}

fn create_track(connection: &SqliteConnection, body: TrackBody) -> Result<TrackEntity, failure::Error> {
    let repository = TrackRepository::new(connection);
    let result = repository.create_entity(body)?;
    Ok(result)
}

fn handle_post_tracks(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity_body: TrackBody = match Headers::take_from(&mut state).get::<ContentType>() {
                    Some(content_type) => {
                        if &content_type.0 == &mime::APPLICATION_JSON {
                            match serde_json::from_slice(&valid_body) {
                                Ok(entity_body) => entity_body,
                                Err(e) => {
                                    return future::err((
                                        state,
                                        e.into_handler_error().with_status(StatusCode::BadRequest),
                                    ))
                                }
                            }
                        } else if &content_type.0 == &mime::APPLICATION_MSGPACK {
                            match rmp_serde::from_slice(&valid_body) {
                                Ok(entity_body) => entity_body,
                                Err(e) => {
                                    return future::err((
                                        state,
                                        e.into_handler_error().with_status(StatusCode::BadRequest),
                                    ))
                                }
                            }
                        } else {
                            let e = format_err!("Unsupported content type");
                            return future::err((
                                state,
                                e.compat().into_handler_error().with_status(StatusCode::UnsupportedMediaType),
                            ))
                        }
                    },
                    None => {
                        let e = format_err!("Missing content type");
                        return future::err((
                            state,
                            e.compat().into_handler_error().with_status(StatusCode::UnsupportedMediaType),
                        ))
                    },
                };

                let connection = &*gotham_middleware_diesel::state_data::connection(&state);

                let entity = match create_track(connection, entity_body) {
                    Ok(entity) => entity,
                    Err(e) => {
                        return future::err((
                            state,
                            e.compat().into_handler_error(),
                        ))
                    }
                };

                let response = match serde_json::to_vec(entity.header()) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => return future::err((state, e.into_handler_error())),
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, e.into_handler_error())),
        });

    Box::new(handler_future)
}

fn load_track(connection: &SqliteConnection, uid: &EntityUid) -> Result<Option<SerializedEntity>, failure::Error> {
    let repository = TrackRepository::new(connection);
    let result = repository.load_entity(&uid)?;
    Ok(result)
}

fn handle_get_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let path = UidPathExtractor::take_from(&mut state);
    let uid: EntityUid = path.uid.into();

    let connection = &*gotham_middleware_diesel::state_data::connection(&state);

    let result = match load_track(connection, &uid) {
        Ok(Some(serialized_entity)) => {
            let response = create_response(
                &state,
                StatusCode::Ok,
                Some((serialized_entity.serialized_blob, serialized_entity.format.into())),
            );
            Ok((state, response))
        },
        Ok(None) => {
            let response = create_response(&state, StatusCode::NotFound, None);
            Ok((state, response))
        }
        Err(e) => Err((state, e.compat().into_handler_error())),
    };

    Box::new(result.into_future())
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
        route.post("/collections").to(handle_post_collections);
        route
            .put("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_put_collections_path_uid);
        route
            .get("/collections")
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_get_collections_path_pagination);
        route
            .get("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_get_collections_path_uid);
        route
            .delete("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_delete_collections_path_uid);
        route.post("/tracks").to(handle_post_tracks);
        route
            .get("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_get_tracks_path_uid);
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
