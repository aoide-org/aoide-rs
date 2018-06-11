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

#[macro_use]
extern crate serde;

extern crate serde_json;

use aoide::{middleware, middleware::DieselMiddleware};

use aoide_storage::{storage::{collections::*,
                              serde::*,
                              tracks::{util::TrackRepositoryHelper, *}},
                    usecases::{api::{LocateParams, Pagination, PaginationLimit,
                                     PaginationOffset, ScoredTagCount, SearchParams,
                                     StringField, StringFieldCounts, TagFacetCount,
                                     TrackReplacementParams, TrackReplacementReport},
                               *}};

use aoide_core::domain::{collection::*, entity::*, track::*};

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

embed_migrations!("aoide-storage/db/migrations/sqlite");

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
    let helper = TrackRepositoryHelper::new(connection);
    connection.transaction::<_, Error, _>(|| helper.cleanup())
}

fn repair_database_storage(connection_pool: &SqliteConnectionPool) -> Result<(), Error> {
    info!("Repairing database storage");
    let collection_prototype = CollectionBody {
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
    fn parse_from(state: &mut State) -> Result<EntityUid, Error> {
        EntityUid::decode_from_str(&Self::take_from(state).uid)
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

#[derive(Debug, Deserialize, StateData, StaticResponseExtender)]
#[serde(rename_all = "camelCase")]
struct DefaultQueryStringExtractor {
    collection_uid: Option<String>,
    with: Option<String>,
    offset: Option<PaginationOffset>,
    limit: Option<PaginationLimit>,
}

impl DefaultQueryStringExtractor {
    pub fn decode_collection_uid(&self) -> Result<Option<CollectionUid>, Error> {
        match self.collection_uid {
            None => Ok(None),
            Some(ref uid) => match CollectionUid::decode_from_str(uid) {
                Ok(uid) => Ok(Some(uid)),
                Err(e) => Err(e),
            },
        }
    }

    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }

    pub fn with_fields<'a>(&'a self) -> Vec<StringField> {
        let mut result = Vec::new();
        if let Some(ref field_list) = self.with {
            result = field_list
                .split(',')
                .map(|field_str| serde_json::from_str(&format!("\"{}\"", field_str)))
                .filter_map(|from_str| from_str.ok())
                .collect();
            debug_assert!(result.len() <= field_list.split(',').count());
            let unrecognized_field_count = field_list.split(',').count() - result.len();
            if unrecognized_field_count > 0 {
                warn!(
                    "{} unrecognized field selector(s) in '{}'",
                    unrecognized_field_count, field_list
                );
            }
            result.sort();
            result.dedup();
        }
        result
    }

    pub fn with_facets<'a>(&'a self) -> Option<Vec<&'a str>> {
        self.with
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

fn create_collection(
    pooled_connection: SqlitePooledConnection,
    body: CollectionBody,
) -> Result<CollectionEntity, Error> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.create_entity(body))
}

fn handle_create_collection(mut state: State) -> Box<HandlerFuture> {
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

fn handle_update_collection(mut state: State) -> Box<HandlerFuture> {
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

fn delete_collection(
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

    let response = match delete_collection(pooled_connection, &uid) {
        Ok(_) => create_response(&state, StatusCode::NoContent, None),
        Err(e) => format_response_message(&state, StatusCode::InternalServerError, &e),
    };

    Box::new(future::ok((state, response)))
}

fn load_collection(
    pooled_connection: SqlitePooledConnection,
    uid: &EntityUid,
    with_track_stats: bool,
) -> Result<Option<CollectionEntity>, Error> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    let mut collection = repository.find_entity(&uid)?;
    if let Some(ref mut collection) = collection {
        if with_track_stats {
            let track_repo = TrackRepository::new(connection);
            debug_assert!(collection.stats.tracks.is_none());
            collection.stats.tracks = Some(track_repo.collection_stats(uid)?);
        }
    }
    Ok(collection)
}

fn handle_load_collection(mut state: State) -> Box<HandlerFuture> {
    let uid = match UidPathExtractor::parse_from(&mut state) {
        Ok(uid) => uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
    };
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match load_collection(
        pooled_connection,
        &uid,
        query_params.try_with_token("track-stats"),
    ) {
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

fn list_collections(
    pooled_connection: SqlitePooledConnection,
    pagination: &Pagination,
    with_track_stats: bool,
) -> Result<Vec<CollectionEntity>, Error> {
    let connection = &*pooled_connection;
    let repository = CollectionRepository::new(connection);
    let mut collections = repository.find_recently_revisioned_entities(pagination)?;
    if with_track_stats {
        let repository = TrackRepository::new(connection);
        for collection in collections.iter_mut() {
            debug_assert!(collection.stats.tracks.is_none());
            collection.stats.tracks = Some(repository.collection_stats(collection.header().uid())?);
        }
    }
    Ok(collections)
}

fn handle_list_collections(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);
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

    let response = match list_collections(
        pooled_connection,
        &pagination,
        query_params.try_with_token("track-stats"),
    ) {
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

fn create_track(
    pooled_connection: SqlitePooledConnection,
    body: TrackBody,
    format: SerializationFormat,
) -> TracksResult<TrackEntity> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.create_entity(body, format))
}

fn handle_create_track(mut state: State) -> Box<HandlerFuture> {
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

fn handle_update_track(mut state: State) -> Box<HandlerFuture> {
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

fn delete_track(pooled_connection: SqlitePooledConnection, uid: &EntityUid) -> Result<(), Error> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    connection.transaction::<_, Error, _>(|| repository.remove_entity(&uid))
}

fn handle_delete_track(mut state: State) -> Box<HandlerFuture> {
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

    let response = match delete_track(pooled_connection, &uid) {
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

fn handle_load_track(mut state: State) -> Box<HandlerFuture> {
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

fn handle_list_tracks(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
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

    let response = match search_tracks(
        pooled_connection,
        collection_uid.as_ref(),
        &query_params.pagination(),
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

fn handle_search_tracks(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
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
                    &query_params.pagination(),
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

fn locate_track(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    pagination: &Pagination,
    locate_params: LocateParams,
) -> TracksResult<Vec<SerializedEntity>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.locate_entities(collection_uid, pagination, locate_params)
}

fn handle_locate_track(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
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

                let response = match locate_track(
                    pooled_connection,
                    collection_uid.as_ref(),
                    &query_params.pagination(),
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

fn handle_replace_tracks(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
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
                    Ok(report) => match serialize_with_format(&report, format) {
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
                    },
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

fn list_fields(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    fields: Vec<StringField>,
    pagination: &Pagination,
) -> TracksResult<Vec<StringFieldCounts>> {
    let mut results: Vec<StringFieldCounts> = Vec::with_capacity(fields.len());

    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    for field in fields.into_iter() {
        let result = repository.list_fields(collection_uid, field, pagination)?;
        results.push(result);
    }

    Ok(results)
}

fn handle_list_fields(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
        Err(e) => {
            let response = format_response_message(&state, StatusCode::BadRequest, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let selected_fields = query_params.with_fields();
    if selected_fields.is_empty() {
        warn!("No countable fields selected");
        let response = create_response(&state, StatusCode::NoContent, None);
        return Box::new(future::ok((state, response)));
    }

    let pooled_connection = match middleware::state_data::try_connection(&state) {
        Ok(pooled_connection) => pooled_connection,
        Err(e) => {
            error!("No database connection: {:?}", &e);
            let response = format_response_message(&state, StatusCode::InternalServerError, &e);
            return Box::new(future::ok((state, response)));
        }
    };

    let response = match list_fields(
        pooled_connection,
        collection_uid.as_ref(),
        selected_fields,
        &query_params.pagination(),
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

fn list_tag_facets(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    facets: Option<&Vec<&str>>,
    pagination: &Pagination,
) -> TrackTagsResult<Vec<TagFacetCount>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.list_tag_facets(collection_uid, facets, pagination)
}

fn handle_list_tag_facets(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
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

    let response = match list_tag_facets(
        pooled_connection,
        collection_uid.as_ref(),
        query_params.with_facets().as_ref(),
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

fn list_tags(
    pooled_connection: SqlitePooledConnection,
    collection_uid: Option<&EntityUid>,
    facets: Option<&Vec<&str>>,
    pagination: &Pagination,
) -> TrackTagsResult<Vec<ScoredTagCount>> {
    let connection = &*pooled_connection;
    let repository = TrackRepository::new(connection);
    repository.list_tags(collection_uid, facets, pagination)
}

fn handle_list_tags(mut state: State) -> Box<HandlerFuture> {
    let query_params = DefaultQueryStringExtractor::take_from(&mut state);

    let collection_uid = match query_params.decode_collection_uid() {
        Ok(collection_uid) => collection_uid,
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

    let response = match list_tags(
        pooled_connection,
        collection_uid.as_ref(),
        query_params.with_facets().as_ref(),
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
        route // load collection
            .get("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_load_collection);
        route // update collection
            .put("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_update_collection);
        route // delete collection
            .delete("/collections/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_delete_collections_path_uid);
        route // list collections
            .get("/collections")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_list_collections);
        route // create collection
            .post("/collections")
            .to(handle_create_collection);
        route // load track
            .get("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_load_track);
        route // update track
            .put("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_update_track);
        route // delete track
            .delete("/tracks/:uid")
            .with_path_extractor::<UidPathExtractor>()
            .to(handle_delete_track);
        route // search tracks
            .post("/tracks/search")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_search_tracks);
        route // locate track
            .post("/tracks/locate")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_locate_track);
        route // replace tracks
            .post("/tracks/replace")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_replace_tracks);
        route // list tracks
            .get("/tracks")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_list_tracks);
        route // create track
            .post("/tracks")
            .to(handle_create_track);
        route // list tag facets
            .get("/tags/facets")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_list_tag_facets);
        route // list tags
            .get("/tags")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_list_tags);
        route // list (string) fields
            .get("/fields")
            .with_query_string_extractor::<DefaultQueryStringExtractor>()
            .to(handle_list_fields);
    })
}

pub fn main() -> Result<(), Error> {
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
