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

extern crate clap;

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

extern crate hyper;

#[macro_use]
extern crate log;

extern crate mime;

extern crate r2d2;

extern crate rmp_serde;

extern crate serde;

extern crate serde_cbor;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

use aoide::{middleware,
            middleware::DieselMiddleware,
            storage::{collections::*, serde::*, tracks::*},
            usecases::{api::{CountableStringField, LocateParams, Pagination, PaginationLimit,
                             PaginationOffset, TrackReplacementParams, TrackReplacementReport, ResourceStats, SearchParams,
                             StringFieldCounts},
                       *}};

use aoide_core::domain::{collection::*, entity::*, metadata::*, track::*};

use clap::{App, Arg};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use failure::Error;

use futures::{future, Future, Stream};
// futures v0.2.1
//use futures::{future, Future};
//use futures::stream::{Stream, StreamExt};

// Gotham v0.3
//use gotham::helpers::http::response::create_response;
use gotham::handler::HandlerFuture;
use gotham::http::response::create_response;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};

use hyper::header::{ContentType, Headers};
use hyper::{Response, StatusCode};

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use r2d2::{Pool, PooledConnection};

use std::env;
use std::str;

embed_migrations!("db/migrations/sqlite");

type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;
type SqliteDieselMiddleware = DieselMiddleware<SqliteConnection>;

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
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.cleanup_aux_storage())
}

fn repair_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    info!("Repairing database storage");
    let collection_prototype = CollectionBody {
        name: "Missing Collection".into(),
        description: Some("Recreated by aoide".into()),
    };
    let connection = &*connection_pool.get()?;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| {
        repository.recreate_missing_collections(&collection_prototype)
    })?;
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

fn create_response_message<S: Into<String>>(
    state: &State,
    response_code: StatusCode,
    response_message: S,
) -> Response {
    let response_text = response_message.into();
    create_response(
        &state,
        response_code,
        Some((response_text.into_bytes(), mime::TEXT_PLAIN_UTF_8)),
    )
}

fn format_response_message<D: std::fmt::Display>(
    state: &State,
    response_code: StatusCode,
    displayable: &D,
) -> Response {
    let response_message = format!("{}", displayable);
    create_response_message(state, response_code, response_message)
}

fn parse_serialization_format_from_state(state: &State) -> Result<SerializationFormat, Error> {
    match Headers::borrow_from(state).get::<ContentType>() {
        Some(content_type) => {
            if let Some(format) = SerializationFormat::from_media_type(&content_type.0) {
                Ok(format)
            } else {
                Err(format_err!("Unsupported content type"))
            }
        }
        None => Err(format_err!("Missing content type")),
    }
}

#[derive(Deserialize, StateData, StaticResponseExtender)]
struct UidPathExtractor {
    uid: String,
}

impl UidPathExtractor {
    fn try_parse_from(state: &mut State) -> Option<EntityUid> {
        Self::try_take_from(state).map(|path| path.uid.into())
    }

    fn parse_from(state: &mut State) -> Result<EntityUid, Error> {
        match Self::try_parse_from(state) {
            Some(uid) => Ok(uid),
            None => {
                let e = format_err!("Missing or invalid identifier");
                Err(e)
            }
        }
    }

    fn parse_from_and_verify(
        state: &mut State,
        expected_uid: &EntityUid,
    ) -> Result<EntityUid, Error> {
        match Self::parse_from(state) {
            Ok(uid) => {
                if &uid == expected_uid {
                    Ok(uid)
                } else {
                    let e = format_err!(
                        "Mismatching identifiers: expected = {}, actual = {}",
                        expected_uid,
                        uid
                    );
                    Err(e)
                }
            }
            Err(e) => Err(e),
        }
    }
}

fn find_collection(
    pooled_connection: SqlitePooledConnection,
    uid: &EntityUid,
) -> CollectionsResult<Option<CollectionEntity>> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    repository.find_entity(&uid)
}

fn handle_get_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match find_collection(pooled_connection, &uid) {
        Ok(Some(collection)) => match serde_json::to_vec(&collection) {
            Ok(response_body) => create_response(
                &state,
                StatusCode::Ok,
                Some((response_body, mime::APPLICATION_JSON)),
            ),
            Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
        },
        Ok(None) => create_response(&state, StatusCode::NotFound, None),
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn remove_collection(
    pooled_connection: SqlitePooledConnection,
    uid: &EntityUid,
) -> CollectionsResult<Option<()>> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.remove_entity(&uid))
}

fn handle_delete_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match remove_collection(pooled_connection, &uid) {
        Ok(_) => create_response(&state, StatusCode::NoContent, None),
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

#[derive(Debug, Deserialize, StateData, StaticResponseExtender)]
struct PaginationQueryStringExtractor {
    offset: Option<PaginationOffset>,
    limit: Option<PaginationLimit>,
}

fn find_recently_revisioned_collections(
    pooled_connection: SqlitePooledConnection,
    pagination: &Pagination,
) -> CollectionsResult<Vec<CollectionEntity>> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    repository.find_recently_revisioned_entities(pagination)
}

fn handle_get_collections_query_pagination(mut state: State) -> Box<HandlerFuture> {
    let query_params = PaginationQueryStringExtractor::take_from(&mut state);
    let pagination = Pagination {
        offset: query_params.offset,
        limit: query_params.limit,
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match find_recently_revisioned_collections(pooled_connection, &pagination) {
        Ok(collections) => match serde_json::to_vec(&collections) {
            Ok(response_body) => create_response(
                &state,
                StatusCode::Ok,
                Some((response_body, mime::APPLICATION_JSON)),
            ),
            Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
        },
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn create_collection(
    pooled_connection: SqlitePooledConnection,
    body: CollectionBody,
) -> Result<CollectionEntity, Error> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.create_entity(body))
}

fn handle_post_collections(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity_body: CollectionBody = match serde_json::from_slice(&valid_body) {
                    Ok(p) => p,
                    Err(e) => {
                        let response = format_response_message(&state, StatusCode::BadRequest, &e);
                        return future::ok((state, response));
                    }
                };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let entity = match create_collection(pooled_connection, entity_body) {
                    Ok(entity) => entity,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match serde_json::to_vec(&entity.header()) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };
                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                return future::ok((state, response));
            }
        });

    Box::new(handler_future)
}

fn update_collection(
    pooled_connection: SqlitePooledConnection,
    entity: &CollectionEntity,
) -> CollectionsResult<Option<(EntityRevision, EntityRevision)>> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.update_entity(entity))
}

fn handle_put_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity: CollectionEntity = match serde_json::from_slice(&valid_body) {
                    Ok(p) => p,
                    Err(e) => {
                        let response = format_response_message(&state, StatusCode::BadRequest, &e);
                        return future::ok((state, response));
                    }
                };

                let uid = match UidPathExtractor::parse_from_and_verify(
                    &mut state,
                    entity.header().uid(),
                ) {
                    Ok(uid) => uid,
                    Err(e) => {
                        let response = format_response_message(&state, StatusCode::BadRequest, &e);
                        return future::ok((state, response));
                    }
                };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let prev_revision = entity.header().revision();
                let next_revision = match update_collection(pooled_connection, &entity) {
                    Ok(Some((_, next_revision))) => next_revision,
                    Ok(None) => {
                        let prev_header = EntityHeader::new(uid, prev_revision);
                        let response_message =
                            format!("Inexistent entity or revision conflict: {:?}", prev_header);
                        let response = create_response_message(
                            &state,
                            StatusCode::InternalServerError,
                            response_message,
                        );
                        return future::ok((state, response));
                    }
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match serde_json::to_vec(&EntityHeader::new(uid, next_revision)) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Ok,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };
                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                return future::ok((state, response));
            }
        });

    Box::new(handler_future)
}

fn create_track(
    pooled_connection: SqlitePooledConnection,
    body: TrackBody,
    format: SerializationFormat,
) -> TracksResult<TrackEntity> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.create_entity(body, format))
}

fn handle_post_tracks(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::UnsupportedMediaType, &e);
                        return future::ok((state, response));
                    }
                };

                let entity_body: TrackBody =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(entity_body) => entity_body,
                        Err(e) => {
                            warn!(
                                "Deserialization failed: {}",
                                str::from_utf8(&valid_body).unwrap()
                            );
                            let response =
                                format_response_message(&state, StatusCode::BadRequest, &e);
                            return future::ok((state, response));
                        }
                    };
                if !entity_body.is_valid() {
                    warn!("Invalid track: {:?}", entity_body);
                }

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let entity = match create_track(pooled_connection, entity_body, format) {
                    Ok(entity) => entity,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match serialize_with_format(entity.header(), format) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };
                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                return future::ok((state, response));
            }
        });

    Box::new(handler_future)
}

fn update_track(
    pooled_connection: SqlitePooledConnection,
    entity: &mut TrackEntity,
    format: SerializationFormat,
) -> TracksResult<Option<(EntityRevision, EntityRevision)>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.update_entity(entity, format))
}

fn handle_put_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::UnsupportedMediaType, &e);
                        return future::ok((state, response));
                    }
                };

                let mut entity: TrackEntity =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(entity_body) => entity_body,
                        Err(e) => {
                            warn!(
                                "Deserialization failed: {}",
                                str::from_utf8(&valid_body).unwrap()
                            );
                            let response =
                                format_response_message(&state, StatusCode::BadRequest, &e);
                            return future::ok((state, response));
                        }
                    };
                if !entity.body().is_valid() {
                    warn!("Invalid track: {:?}", entity.body());
                }

                let uid = match UidPathExtractor::parse_from_and_verify(
                    &mut state,
                    entity.header().uid(),
                ) {
                    Ok(uid) => uid,
                    Err(e) => {
                        let response = format_response_message(&state, StatusCode::BadRequest, &e);
                        return future::ok((state, response));
                    }
                };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let prev_revision = entity.header().revision();
                let next_revision = match update_track(pooled_connection, &mut entity, format) {
                    Ok(Some((_, next_revision))) => {
                        debug_assert!(next_revision == entity.header().revision());
                        next_revision
                    }
                    Ok(None) => {
                        let prev_header = EntityHeader::new(uid, prev_revision);
                        let response_message =
                            format!("Inexistent entity or revision conflict: {:?}", prev_header);
                        let response =
                            create_response_message(&state, StatusCode::NotFound, response_message);
                        return future::ok((state, response));
                    }
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match serialize_with_format(
                    &EntityHeader::new(uid, next_revision),
                    format,
                ) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Ok,
                        Some((response_body, format.into())),
                    ),
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };
                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                return future::ok((state, response));
            }
        });

    Box::new(handler_future)
}

fn remove_track(pooled_connection: SqlitePooledConnection, uid: &EntityUid) -> Result<(), Error> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.remove_entity(&uid))
}

fn handle_delete_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match remove_track(pooled_connection, &uid) {
        Ok(_) => create_response(&state, StatusCode::NoContent, None),
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn load_track(
    pooled_connection: SqlitePooledConnection,
    uid: &EntityUid,
) -> TracksResult<Option<SerializedEntity>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.load_entity(&uid)
}

fn handle_get_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match load_track(pooled_connection, &uid) {
        Ok(Some(serialized_entity)) => create_response(
            &state,
            StatusCode::Ok,
            Some((serialized_entity.blob, serialized_entity.format.into())),
        ),
        Ok(None) => create_response(&state, StatusCode::NotFound, None),
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn locate_tracks(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    pagination: &Pagination,
    locate_params: LocateParams,
) -> TracksResult<Vec<SerializedEntity>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.locate_entities(collection_uid, pagination, locate_params)
}

fn handle_post_collections_path_uid_tracks_locate_query_pagination(
    mut state: State,
) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);
    let query_params = PaginationQueryStringExtractor::take_from(&mut state);
    let pagination = Pagination {
        offset: query_params.offset,
        limit: query_params.limit,
    };

    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::UnsupportedMediaType, &e);
                        return future::ok((state, response));
                    }
                };

                let locate_params: LocateParams =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(locate_params) => locate_params,
                        Err(e) => {
                            warn!(
                                "Deserialization failed: {}",
                                str::from_utf8(&valid_body).unwrap()
                            );
                            let response =
                                format_response_message(&state, StatusCode::BadRequest, &e);
                            return future::ok((state, response));
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match locate_tracks(
                    pooled_connection,
                    collection_uid.as_ref(),
                    &pagination,
                    locate_params,
                ).and_then(concat_serialized_entities_into_json_array)
                {
                    Ok(json_array) => {
                        create_response(&state, StatusCode::Ok, Some((json_array, format.into())))
                    }
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };
                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                return future::ok((state, response));
            }
        });

    Box::new(handler_future)
}

fn replace_tracks(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    replacement_params: TrackReplacementParams,
    format: SerializationFormat,
) -> TracksResult<TrackReplacementReport> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| {
        repository.replace_entities(collection_uid, replacement_params, format)
    })
}

fn handle_post_collections_path_uid_tracks_replace(mut state: State) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);

    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::UnsupportedMediaType, &e);
                        return future::ok((state, response));
                    }
                };

                let replacement_params: TrackReplacementParams =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(replacement_params) => replacement_params,
                        Err(e) => {
                            warn!(
                                "Deserialization failed: {}",
                                str::from_utf8(&valid_body).unwrap()
                            );
                            let response =
                                format_response_message(&state, StatusCode::BadRequest, &e);
                            return future::ok((state, response));
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match replace_tracks(
                    pooled_connection,
                    collection_uid.as_ref(),
                    replacement_params,
                    format,
                ) {
                    Ok(report) => {
                        match serialize_with_format(&report, format) {
                            Ok(response_body) => create_response(
                                &state,
                                StatusCode::Ok,
                                Some((response_body, format.into())),
                            ),
                            Err(e) => {
                                let response = format_response_message(
                                    &state,
                                    StatusCode::InternalServerError,
                                    &e,
                                );
                                return future::ok((state, response));
                            }
                        }
                    }
                    Err(e) => {
                        let response = format_response_message(&state, StatusCode::BadRequest, &e);
                        return future::ok((state, response));
                    }
                };

                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                return future::ok((state, response));
            }
        });

    Box::new(handler_future)
}

fn search_tracks(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    pagination: &Pagination,
    search_params: SearchParams,
) -> TracksResult<Vec<SerializedEntity>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.search_entities(collection_uid, pagination, search_params)
}

fn handle_post_collections_path_uid_tracks_search_query_pagination(
    mut state: State,
) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);
    let query_params = PaginationQueryStringExtractor::take_from(&mut state);
    let pagination = Pagination {
        offset: query_params.offset,
        limit: query_params.limit,
    };

    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        let response =
                            format_response_message(&state, StatusCode::UnsupportedMediaType, &e);
                        return future::ok((state, response));
                    }
                };

                let search_params: SearchParams =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(search_params) => search_params,
                        Err(e) => {
                            warn!(
                                "Deserialization failed: {}",
                                str::from_utf8(&valid_body).unwrap()
                            );
                            let response =
                                format_response_message(&state, StatusCode::BadRequest, &e);
                            return future::ok((state, response));
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => {
                        error!("No database connection: {:?}", &e);
                        let response =
                            format_response_message(&state, StatusCode::InternalServerError, &e);
                        return future::ok((state, response));
                    }
                };

                let response = match search_tracks(
                    pooled_connection,
                    collection_uid.as_ref(),
                    &pagination,
                    search_params,
                ).and_then(concat_serialized_entities_into_json_array)
                {
                    Ok(json_array) => {
                        create_response(&state, StatusCode::Ok, Some((json_array, format.into())))
                    }
                    Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
                };
                future::ok((state, response))
            }
            Err(e) => {
                let response = format_response_message(&state, StatusCode::InternalServerError, &e);
                future::ok((state, response))
            }
        });

    Box::new(handler_future)
}

fn handle_get_collections_path_uid_tracks_query_pagination(mut state: State) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);
    let query_params = PaginationQueryStringExtractor::take_from(&mut state);
    let pagination = Pagination {
        offset: query_params.offset,
        limit: query_params.limit,
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match search_tracks(
        pooled_connection,
        collection_uid.as_ref(),
        &pagination,
        SearchParams::default(),
    ).and_then(concat_serialized_entities_into_json_array)
    {
        Ok(json_array) => create_response(
            &state,
            StatusCode::Ok,
            Some((json_array, mime::APPLICATION_JSON)),
        ),
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn resource_statistics(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
) -> TracksResult<ResourceStats> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.resource_statistics(collection_uid)
}

fn handle_get_collections_path_tracks_stats(mut state: State) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match resource_statistics(pooled_connection, collection_uid.as_ref()) {
        Ok(result) => match serde_json::to_vec(&result) {
            Ok(json) => {
                create_response(&state, StatusCode::Ok, Some((json, mime::APPLICATION_JSON)))
            }
            Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
        },
        Err(e) => {
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    Box::new(future::ok((state, response)))
}

#[derive(Debug, Deserialize, StateData, StaticResponseExtender)]
struct CountableStringFieldQueryStringExtractor {
    field: Option<String>,
}

impl CountableStringFieldQueryStringExtractor {
    pub fn fields<'a>(&'a self) -> Vec<CountableStringField> {
        if let Some(ref field_list) = self.field {
            let mut result: Vec<CountableStringField> = field_list
                .split(',')
                .map(|field_str| serde_json::from_str(&format!("\"{}\"", field_str)))
                .filter_map(|from_str| from_str.ok())
                .collect();
            debug_assert!(result.len() <= field_list.split(',').count());
            let unrecognized_field_count = field_list.split(',').count() - result.len();
            if unrecognized_field_count > 0 {
                warn!(
                    "{} unrecognized field(s) in '{}'",
                    unrecognized_field_count, field_list
                );
            }
            result.sort();
            result.dedup();
            result
        } else {
            // All of the following fields if the corresponding query
            // parameter is missing.
            // This functionality is only provided for convenience and
            // for testing. It is recommended to specify all required
            // fields explicitly.
            vec![
                CountableStringField::AlbumArtist,
                CountableStringField::AlbumTitle,
                CountableStringField::TrackArtist,
                CountableStringField::TrackTitle,
            ]
        }
    }
}

fn all_field_counts(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    fields: Vec<CountableStringField>,
) -> TracksResult<Vec<StringFieldCounts>> {
    let mut results: Vec<StringFieldCounts> = Vec::with_capacity(fields.len());

    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    for field in fields.into_iter() {
        let result = repository.field_counts(collection_uid, field)?;
        results.push(result);
    }

    Ok(results)
}

fn handle_get_collections_path_uid_tracks_fields_query_field(
    mut state: State,
) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);
    let query_params = CountableStringFieldQueryStringExtractor::take_from(&mut state);

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match all_field_counts(
        pooled_connection,
        collection_uid.as_ref(),
        query_params.fields(),
    ) {
        Ok(results) => match serde_json::to_vec(&results) {
            Ok(json) => {
                create_response(&state, StatusCode::Ok, Some((json, mime::APPLICATION_JSON)))
            }
            Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
        },
        Err(e) => {
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    Box::new(future::ok((state, response)))
}

#[derive(Debug, Deserialize, StateData, StaticResponseExtender)]
struct TagFacetPaginationQueryStringExtractor {
    facet: Option<String>,
    offset: Option<PaginationOffset>,
    limit: Option<PaginationLimit>,
}

impl TagFacetPaginationQueryStringExtractor {
    pub fn facets<'a>(&'a self) -> Option<Vec<&'a str>> {
        self.facet
            .as_ref()
            .map(|facet_list| facet_list.split(',').collect::<Vec<&'a str>>())
            .map(|mut facets| {
                facets.sort();
                facets
            })
            .map(|mut facets| {
                facets.dedup();
                facets
            })
    }

    pub fn pagination(&self) -> Pagination {
        Pagination {
            offset: self.offset,
            limit: self.limit,
        }
    }
}

fn all_tags_facets(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    facets: Option<&Vec<&str>>,
    pagination: &Pagination,
) -> TrackTagsResult<Vec<TagFacetCount>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.all_tags_facets(collection_uid, facets, pagination)
}

fn handle_get_collections_path_uid_tracks_tags_facets_query_facet_pagination(
    mut state: State,
) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);
    let query_params = TagFacetPaginationQueryStringExtractor::take_from(&mut state);

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match all_tags_facets(
        pooled_connection,
        collection_uid.as_ref(),
        query_params.facets().as_ref(),
        &query_params.pagination(),
    ) {
        Ok(result) => match serde_json::to_vec(&result) {
            Ok(response_body) => create_response(
                &state,
                StatusCode::Ok,
                Some((response_body, mime::APPLICATION_JSON)),
            ),
            Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
        },
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn all_tags(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    facets: Option<&Vec<&str>>,
    pagination: &Pagination,
) -> TrackTagsResult<Vec<MultiTag>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.all_tags(collection_uid, facets, pagination)
}

fn handle_get_collections_path_uid_tracks_tags_query_facet_pagination(
    mut state: State,
) -> Box<HandlerFuture> {
    let collection_uid = UidPathExtractor::try_parse_from(&mut state);
    let query_params = TagFacetPaginationQueryStringExtractor::take_from(&mut state);

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match all_tags(
        pooled_connection,
        collection_uid.as_ref(),
        query_params.facets().as_ref(),
        &query_params.pagination(),
    ) {
        Ok(result) => match serde_json::to_vec(&result) {
            Ok(response_body) => create_response(
                &state,
                StatusCode::Ok,
                Some((response_body, mime::APPLICATION_JSON)),
            ),
            Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
        },
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
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
        route // add single collection (body)
            .post("/collections")
            .to(handle_post_collections);
        route // update single collection
            .put("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_put_collections_path_uid);
        route // remove single collection
            .delete("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_delete_collections_path_uid);
        route // load single collection
            .get("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_get_collections_path_uid);
        route // load recently modified collections
            .get("/collections")
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_get_collections_query_pagination);
        route // add single track (body)
            .post("/tracks")
            .to(handle_post_tracks);
        route // update single track
            .put("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_put_tracks_path_uid);
        route // remove single track
            .delete("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_delete_tracks_path_uid);
        route // load single track
            .get("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_get_tracks_path_uid);
        route // load recently modified tracks
            .get("/tracks")
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_get_collections_path_uid_tracks_query_pagination);
        route // load recently modified tracks from collection
            .get("/collections/:uid/tracks")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_get_collections_path_uid_tracks_query_pagination);
        route // locate multiple track in collection
            .post("/collections/:uid/tracks/locate")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_post_collections_path_uid_tracks_locate_query_pagination);
        route // replace single track in collection
            .post("/collections/:uid/tracks/replace")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_post_collections_path_uid_tracks_replace);
        route // search multiple tracks in collection
            .post("/collections/:uid/tracks/search")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_post_collections_path_uid_tracks_search_query_pagination);
        route // various statistics about tracks in collection
            .get("/collections/:uid/tracks/stats")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_get_collections_path_tracks_stats);
        route // selected (string) fields in collection
            .get("/collections/:uid/tracks/fields")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<CountableStringFieldQueryStringExtractor>()
            .to(handle_get_collections_path_uid_tracks_fields_query_field);
        route // all tag facets in collection
            .get("/collections/:uid/tracks/tags/facets")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<TagFacetPaginationQueryStringExtractor>()
            .to(handle_get_collections_path_uid_tracks_tags_facets_query_facet_pagination);
        route // all tag (facet, term) tuples in collection
            .get("/collections/:uid/tracks/tags")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<TagFacetPaginationQueryStringExtractor>()
            .to(handle_get_collections_path_uid_tracks_tags_query_facet_pagination);
    })
}

pub fn main() -> Result<(), failure::Error> {
    let matches = App::new("aoide")
            .version("0.0.1")
            //.author("")
            //.about("")
            .arg(Arg::with_name("DB_URL")
                .help("Sets the database URL")
                .default_value(":memory:")
                .index(1))
            .arg(Arg::with_name("LISTEN_ADDR")
                .short("l")
                .long("listen")
                .default_value("localhost:7878")
                .help("Sets the network listen address"))
            .arg(Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity (= number of occurrences)"))
            .get_matches();

    let verbosity = matches.occurrences_of("verbosity");
    init_env_logger_verbosity(verbosity.min(8) as u8);

    let db_url = matches.value_of("DB_URL").unwrap();
    info!("Database URL: {}", db_url);

    let listen_addr = matches.value_of("LISTEN_ADDR").unwrap();
    info!("Network listen address: {}", listen_addr);

    // Workaround: Use a pool of size 1 to avoid 'database is locked'
    // errors due to multi-threading.
    let connection_pool =
        create_connection_pool(db_url, 1).expect("Failed to create database connection pool");

    migrate_database_schema(&connection_pool).unwrap();

    cleanup_database_storage(&connection_pool).unwrap();

    repair_database_storage(&connection_pool).unwrap();

    info!("Creating middleware");
    let middleware = DieselMiddleware::with_pool(connection_pool);

    info!("Creating router");
    let router = router(middleware);

    info!("Listening for requests at http://{}", listen_addr);
    gotham::start(listen_addr, router);

    info!("Exiting");
    Ok(())
}
