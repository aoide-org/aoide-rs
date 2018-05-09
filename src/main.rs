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

extern crate hyper;

#[macro_use]
extern crate log;

extern crate mime;

extern crate r2d2;

extern crate r2d2_diesel;

extern crate rmp_serde;

extern crate serde;

extern crate serde_cbor;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

use aoide_core::domain::collection::*;
use aoide_core::domain::track::*;
use aoide_core::domain::entity::*;
use aoide::middleware;
use aoide::middleware::DieselMiddleware;
use aoide::storage::collections::*;
use aoide::storage::tracks::*;
use aoide::storage::serde::*;
use aoide::usecases::*;
use aoide::usecases::request::{LocateParams, ReplaceParams, SearchParams};
use aoide::usecases::result::*;

use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::result::DatabaseErrorKind;

use futures::{future, Future, Stream};
use futures::future::IntoFuture;
// futures v0.2.1
//use futures::{future, Future};
//use futures::stream::{Stream, StreamExt};

// Gotham v0.3
//use gotham::helpers::http::response::create_response;
use gotham::http::response::create_response;
use gotham::router::Router;
use gotham::router::builder::*;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::set::{finalize_pipeline_set, new_pipeline_set};
use gotham::state::{FromState, State};
use gotham::handler::{HandlerError, HandlerFuture, IntoHandlerError};

use hyper::StatusCode;
use hyper::header::{ContentType, Headers};

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use std::env;
use std::error;

embed_migrations!("db/migrations/sqlite");

type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
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

fn cleanup_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), failure::Error> {
    info!("Cleaning up database storage");
    let pooled_connection = connection_pool.get()?;
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.cleanup_aux_storage())
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

fn on_handler_error<T>(e: T) -> HandlerError
where
    T: error::Error + Send + 'static,
{
    warn!("Failed to handle request: {}", e);
    if log_enabled!(log::Level::Debug) {
        debug!("Error: {:?}", e);
    }
    e.into_handler_error()
}

fn on_handler_failure(e: failure::Error) -> HandlerError {
    match e.cause().downcast_ref::<DieselError>() {
        Some(&DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _))
        | Some(&DieselError::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, _)) => {
            on_handler_error(e.compat()).with_status(StatusCode::BadRequest)
        }
        _ => on_handler_error(e.compat()),
    }
}

fn parse_serialization_format_from_state(
    state: &State,
) -> Result<SerializationFormat, failure::Error> {
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

    fn parse_from(state: &mut State) -> Result<EntityUid, failure::Error> {
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
    ) -> Result<EntityUid, failure::Error> {
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
    connection: &SqliteConnection,
    uid: &EntityUid,
) -> CollectionsResult<Option<CollectionEntity>> {
    let repository = CollectionRepository::new(connection);
    let result = repository.find_entity(&uid)?;
    Ok(result)
}

fn handle_get_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            return Box::new(future::err((
                state,
                on_handler_failure(e).with_status(StatusCode::BadRequest),
            )))
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => return Box::new(future::err((state, on_handler_error(e)))),
    };

    let result = match find_collection(&*pooled_connection, &uid) {
        Ok(Some(collection)) => match serde_json::to_vec(&collection) {
            Ok(response_body) => {
                let response = create_response(
                    &state,
                    StatusCode::Ok,
                    Some((response_body, mime::APPLICATION_JSON)),
                );
                Ok((state, response))
            }
            Err(e) => Err((state, on_handler_error(e))),
        },
        Ok(None) => {
            let response = create_response(&state, StatusCode::NotFound, None);
            Ok((state, response))
        }
        Err(e) => Err((state, on_handler_failure(e))),
    };

    Box::new(result.into_future())
}

fn remove_collection(
    connection: &SqliteConnection,
    uid: &EntityUid,
) -> CollectionsResult<Option<()>> {
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.remove_entity(&uid))
}

fn handle_delete_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            return Box::new(future::err((
                state,
                on_handler_failure(e).with_status(StatusCode::BadRequest),
            )))
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => return Box::new(future::err((state, on_handler_error(e)))),
    };

    let result = match remove_collection(&*pooled_connection, &uid) {
        Ok(_) => {
            let response = create_response(&state, StatusCode::NoContent, None);
            future::ok((state, response))
        }
        Err(e) => future::err((state, on_handler_failure(e))),
    };

    Box::new(result.into_future())
}

#[derive(Debug, Deserialize, StateData, StaticResponseExtender)]
struct PaginationQueryStringExtractor {
    offset: Option<PaginationOffset>,
    limit: Option<PaginationLimit>,
}

fn find_recently_revisioned_collections(
    connection: &SqliteConnection,
    pagination: &Pagination,
) -> CollectionsResult<Vec<CollectionEntity>> {
    let repository = CollectionRepository::new(connection);
    let result = repository.find_recently_revisioned_entities(pagination)?;
    Ok(result)
}

fn handle_get_collections_query_pagination(mut state: State) -> Box<HandlerFuture> {
    let query_params = PaginationQueryStringExtractor::take_from(&mut state);
    let pagination = Pagination {
        offset: query_params.offset,
        limit: query_params.limit,
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => return Box::new(future::err((state, on_handler_error(e)))),
    };

    let handler_future =
        match find_recently_revisioned_collections(&*pooled_connection, &pagination) {
            Ok(collections) => match serde_json::to_vec(&collections) {
                Ok(response_body) => {
                    let response = create_response(
                        &state,
                        StatusCode::Ok,
                        Some((response_body, mime::APPLICATION_JSON)),
                    );
                    future::ok((state, response))
                }
                Err(e) => future::err((state, on_handler_error(e))),
            },
            Err(e) => future::err((state, on_handler_failure(e))),
        };

    Box::new(handler_future)
}

fn create_collection(
    connection: &SqliteConnection,
    body: CollectionBody,
) -> Result<CollectionEntity, failure::Error> {
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.create_entity(body))
}

fn handle_post_collections(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity_body: CollectionBody = match serde_json::from_slice(&valid_body) {
                    Ok(p) => p,
                    Err(e) => {
                        return future::err((
                            state,
                            on_handler_error(e).with_status(StatusCode::BadRequest),
                        ))
                    }
                };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let entity = match create_collection(&*pooled_connection, entity_body) {
                    Ok(entity) => entity,
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };

                let response = match serde_json::to_vec(&entity.header()) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => return future::err((state, on_handler_error(e))),
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, on_handler_error(e))),
        });

    Box::new(handler_future)
}

fn update_collection(
    connection: &SqliteConnection,
    entity: &CollectionEntity,
) -> CollectionsResult<Option<EntityRevision>> {
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.update_entity(entity))
}

fn handle_put_collections_path_uid(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let entity: CollectionEntity = match serde_json::from_slice(&valid_body) {
                    Ok(p) => p,
                    Err(e) => {
                        return future::err((
                            state,
                            on_handler_error(e).with_status(StatusCode::BadRequest),
                        ));
                    }
                };

                let uid = match UidPathExtractor::parse_from_and_verify(
                    &mut state,
                    entity.header().uid(),
                ) {
                    Ok(uid) => uid,
                    Err(e) => {
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::BadRequest),
                        ))
                    }
                };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let next_revision = match update_collection(&*pooled_connection, &entity) {
                    Ok(Some(next_revision)) => next_revision,
                    Ok(None) => {
                        let e = format_err!(
                            "Inexistent entity or revision conflict: {:?}",
                            entity.header()
                        );
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::NotFound),
                        ));
                    }
                    Err(e) => {
                        return future::err((state, on_handler_failure(e)));
                    }
                };

                let response = match serde_json::to_vec(&EntityHeader::new(uid, next_revision)) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Ok,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => {
                        return future::err((state, on_handler_error(e)));
                    }
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, on_handler_error(e))),
        });

    Box::new(handler_future)
}

fn create_track(
    connection: &SqliteConnection,
    body: TrackBody,
    format: SerializationFormat,
) -> TracksResult<TrackEntity> {
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.create_entity(body, format))
}

fn handle_post_tracks(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::UnsupportedMediaType),
                        ))
                    }
                };

                let entity_body: TrackBody =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(entity_body) => entity_body,
                        Err(e) => {
                            return future::err((
                                state,
                                on_handler_failure(e).with_status(StatusCode::BadRequest),
                            ))
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let entity = match create_track(&*pooled_connection, entity_body, format) {
                    Ok(entity) => entity,
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };

                let response = match serialize_with_format(entity.header(), format) {
                    Ok(response_body) => create_response(
                        &state,
                        StatusCode::Created,
                        Some((response_body, mime::APPLICATION_JSON)),
                    ),
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, on_handler_error(e))),
        });

    Box::new(handler_future)
}

fn update_track(
    connection: &SqliteConnection,
    entity: &mut TrackEntity,
    format: SerializationFormat,
) -> TracksResult<Option<(EntityRevision, EntityRevision)>> {
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.update_entity(entity, format))
}

fn handle_put_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let handler_future = hyper::Body::take_from(&mut state)
        .concat2()
        .then(move |full_body| match full_body {
            Ok(valid_body) => {
                let format = match parse_serialization_format_from_state(&state) {
                    Ok(format) => format,
                    Err(e) => {
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::UnsupportedMediaType),
                        ))
                    }
                };

                let mut entity: TrackEntity =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(entity_body) => entity_body,
                        Err(e) => {
                            return future::err((
                                state,
                                on_handler_failure(e).with_status(StatusCode::BadRequest),
                            ))
                        }
                    };

                let uid = match UidPathExtractor::parse_from_and_verify(
                    &mut state,
                    entity.header().uid(),
                ) {
                    Ok(uid) => uid,
                    Err(e) => {
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::BadRequest),
                        ))
                    }
                };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let prev_revision = entity.header().revision();
                let next_revision = match update_track(&*pooled_connection, &mut entity, format) {
                    Ok(Some((_, next_revision))) => {
                        assert!(next_revision == entity.header().revision());
                        next_revision
                    }
                    Ok(None) => {
                        let prev_header = EntityHeader::new(uid, prev_revision);
                        let e = format_err!(
                            "Inexistent entity or revision conflict: {:?}",
                            prev_header
                        );
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::NotFound),
                        ));
                    }
                    Err(e) => {
                        return future::err((state, on_handler_failure(e)));
                    }
                };

                let response =
                    match serialize_with_format(&EntityHeader::new(uid, next_revision), format) {
                        Ok(response_body) => create_response(
                            &state,
                            StatusCode::Ok,
                            Some((response_body, format.into())),
                        ),
                        Err(e) => {
                            return future::err((state, on_handler_failure(e)));
                        }
                    };
                future::ok((state, response))
            }
            Err(e) => future::err((state, on_handler_error(e))),
        });

    Box::new(handler_future)
}

fn remove_track(connection: &SqliteConnection, uid: &EntityUid) -> Result<(), failure::Error> {
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.remove_entity(&uid))
}

fn handle_delete_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            return Box::new(future::err((
                state,
                on_handler_failure(e).with_status(StatusCode::BadRequest),
            )))
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => return Box::new(future::err((state, on_handler_error(e)))),
    };

    let result = match remove_track(&*pooled_connection, &uid) {
        Ok(_) => {
            let response = create_response(&state, StatusCode::NoContent, None);
            future::ok((state, response))
        }
        Err(e) => future::err((state, on_handler_failure(e))),
    };

    Box::new(result.into_future())
}

fn load_track(
    connection: &SqliteConnection,
    uid: &EntityUid,
) -> TracksResult<Option<SerializedEntity>> {
    let repository = TrackRepository::new(connection);
    let result = repository.load_entity(&uid)?;
    Ok(result)
}

fn handle_get_tracks_path_uid(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            return Box::new(future::err((
                state,
                on_handler_failure(e).with_status(StatusCode::BadRequest),
            )))
        }
    };

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => return Box::new(future::err((state, on_handler_error(e)))),
    };

    let result = match load_track(&*pooled_connection, &uid) {
        Ok(Some(serialized_entity)) => {
            let response = create_response(
                &state,
                StatusCode::Ok,
                Some((serialized_entity.blob, serialized_entity.format.into())),
            );
            Ok((state, response))
        }
        Ok(None) => {
            let response = create_response(&state, StatusCode::NotFound, None);
            Ok((state, response))
        }
        Err(e) => Err((state, on_handler_failure(e))),
    };

    Box::new(result.into_future())
}

fn locate_tracks(
    connection: &SqliteConnection,
    collection_uid: Option<&EntityUid>,
    pagination: &Pagination,
    locate_params: LocateParams,
) -> TracksResult<Vec<SerializedEntity>> {
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
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::UnsupportedMediaType),
                        ))
                    }
                };

                let locate_params: LocateParams =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(locate_params) => locate_params,
                        Err(e) => {
                            return future::err((
                                state,
                                on_handler_failure(e).with_status(StatusCode::BadRequest),
                            ))
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let response = match locate_tracks(
                    &*pooled_connection,
                    collection_uid.as_ref(),
                    &pagination,
                    locate_params,
                ).and_then(concat_serialized_entities_into_json_array)
                {
                    Ok(json_array) => {
                        create_response(&state, StatusCode::Ok, Some((json_array, format.into())))
                    }
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };
                future::ok((state, response))
            }
            Err(e) => future::err((state, on_handler_error(e))),
        });

    Box::new(handler_future)
}

fn replace_track(
    connection: &SqliteConnection,
    collection_uid: Option<&EntityUid>,
    replace_params: ReplaceParams,
    format: SerializationFormat,
) -> TracksResult<TrackEntityReplacement> {
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, failure::Error, _>(|| repository.replace_entity(collection_uid, replace_params, format))
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
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::UnsupportedMediaType),
                        ))
                    }
                };

                let replace_params: ReplaceParams =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(replace_params) => replace_params,
                        Err(e) => {
                            return future::err((
                                state,
                                on_handler_failure(e).with_status(StatusCode::BadRequest),
                            ))
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let (entity, status_code) = match replace_track(
                    &*pooled_connection,
                    collection_uid.as_ref(),
                    replace_params,
                    format,
                ) {
                    Ok(TrackEntityReplacement::Updated(entity)) => (entity, StatusCode::Ok),
                    Ok(TrackEntityReplacement::Created(entity)) => (entity, StatusCode::Created),
                    Ok(TrackEntityReplacement::NotFound) => {
                        let response = create_response(&state, StatusCode::NotFound, None);
                        return future::ok((state, response))
                    }
                    Ok(TrackEntityReplacement::FoundTooMany) => {
                        let response = create_response(&state, StatusCode::BadRequest, None);
                        return future::ok((state, response))
                    }
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };

                let response = match serialize_with_format(entity.header(), format) {
                    Ok(response_body) => create_response(
                        &state,
                        status_code,
                        Some((response_body, format.into())),
                    ),
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };
                future::ok((state, response))
            }
            Err(e) => return future::err((state, on_handler_error(e))),
        });

    Box::new(handler_future)
}

fn search_tracks(
    connection: &SqliteConnection,
    collection_uid: Option<&EntityUid>,
    pagination: &Pagination,
    search_params: SearchParams,
) -> TracksResult<Vec<SerializedEntity>> {
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
                        return future::err((
                            state,
                            on_handler_failure(e).with_status(StatusCode::UnsupportedMediaType),
                        ))
                    }
                };

                let search_params: SearchParams =
                    match deserialize_slice_with_format(&valid_body, format) {
                        Ok(search_params) => search_params,
                        Err(e) => {
                            return future::err((
                                state,
                                on_handler_failure(e).with_status(StatusCode::BadRequest),
                            ))
                        }
                    };

                let pooled_connection = match middleware::state_data::try_connection(&state) {
                    Ok(pooled_connection) => pooled_connection,
                    Err(e) => return future::err((state, on_handler_error(e))),
                };

                let response = match search_tracks(
                    &*pooled_connection,
                    collection_uid.as_ref(),
                    &pagination,
                    search_params,
                ).and_then(concat_serialized_entities_into_json_array)
                {
                    Ok(json_array) => {
                        create_response(&state, StatusCode::Ok, Some((json_array, format.into())))
                    }
                    Err(e) => return future::err((state, on_handler_failure(e))),
                };
                future::ok((state, response))
            }
            Err(e) => return future::err((state, on_handler_error(e))),
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
        Err(e) => return Box::new(future::err((state, on_handler_error(e)))),
    };

    let handler_future = match search_tracks(
        &*pooled_connection,
        collection_uid.as_ref(),
        &pagination,
        SearchParams::default(),
    ).and_then(concat_serialized_entities_into_json_array)
    {
        Ok(json_array) => {
            let response = create_response(
                &state,
                StatusCode::Ok,
                Some((json_array, mime::APPLICATION_JSON)),
            );
            future::ok((state, response))
        }
        Err(e) => future::err((state, on_handler_failure(e))),
    };

    Box::new(handler_future)
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
        route // locate multiple track in (optional) collection
            .post("/collections/:uid/tracks/locate")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_post_collections_path_uid_tracks_locate_query_pagination);
        route // replace single track in (optional) collection
            .post("/collections/:uid/tracks/replace")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_post_collections_path_uid_tracks_replace);
        route // search multiple tracks in (optional) collection
            .post("/collections/:uid/tracks/search")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<PaginationQueryStringExtractor>()
            .to(handle_post_collections_path_uid_tracks_search_query_pagination);
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

    let connection_pool =
        create_connection_pool(db_url).expect("Failed to create database connection pool");

    migrate_database_schema(&connection_pool).unwrap();

    cleanup_database_storage(&connection_pool).unwrap();

    info!("Creating middleware");
    let middleware = DieselMiddleware::with_pool(connection_pool);

    info!("Creating router");
    let router = router(middleware);

    let listen_addr = "127.0.0.1:7878";
    info!("Listening for requests at http://{}", listen_addr);
    gotham::start(listen_addr, router)
}
