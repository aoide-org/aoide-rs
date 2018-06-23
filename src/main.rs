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

#[macro_use]
extern crate failure;

extern crate futures;

extern crate hyper;

#[macro_use]
extern crate log;

extern crate mime;

extern crate r2d2;

#[macro_use]
extern crate serde;

extern crate serde_json;

use aoide_storage::{
    storage::{
        collection::*, serde::*, track::{util::TrackRepositoryHelper, *},
    },
    usecases::{
        api::{
            LocateTracksParams, Pagination, ReplaceTracksParams, ReplacedTracks, ScoredTagCount,
            SearchTracksParams, StringField, StringFieldCounts, TagFacetCount,
        },
        *,
    },
};

use aoide_core::domain::{collection::*, entity::*, track::*};

use actix::prelude::*;
use actix_web::*;

use clap::{App, Arg};

use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;

use failure::Error;

use futures::future::Future;

use env_logger::Builder as LoggerBuilder;

use log::LevelFilter as LogLevelFilter;

use r2d2::{Pool, PooledConnection};

use std::env;
use std::str;

embed_migrations!("aoide-storage/db/migrations/sqlite");

type SqliteConnectionPool = Pool<ConnectionManager<SqliteConnection>>;
type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

pub struct SqliteExecutor {
    pool: SqliteConnectionPool,
}

impl SqliteExecutor {
    pub fn new(pool: SqliteConnectionPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqliteConnectionPool {
        &self.pool
    }

    pub fn pooled_connection(&self) -> Result<SqlitePooledConnection, Error> {
        let pooled_connection = self.pool.get()?;
        Ok(pooled_connection)
    }
}

impl Actor for SqliteExecutor {
    type Context = SyncContext<Self>;
}

struct AppState {
    executor: Addr<Syn, SqliteExecutor>,
}

#[derive(Debug)]
pub struct CreateCollectionMessage {
    pub collection: Collection,
}

pub type CreateCollectionResult = CollectionsResult<CollectionEntity>;

impl Message for CreateCollectionMessage {
    type Result = CreateCollectionResult;
}

impl Handler<CreateCollectionMessage> for SqliteExecutor {
    type Result = CreateCollectionResult;

    fn handle(&mut self, msg: CreateCollectionMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = CollectionRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.create_entity(msg.collection))
    }
}

fn on_create_collection(
    (state, body): (State<AppState>, Json<Collection>),
) -> FutureResponse<HttpResponse> {
    let msg = CreateCollectionMessage {
        collection: body.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(collection) => Ok(HttpResponse::Created().json(collection.header())),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct UpdateCollectionMessage {
    pub collection: CollectionEntity,
}

pub type UpdateCollectionResult = CollectionsResult<(EntityRevision, Option<EntityRevision>)>;

impl Message for UpdateCollectionMessage {
    type Result = UpdateCollectionResult;
}

impl Handler<UpdateCollectionMessage> for SqliteExecutor {
    type Result = UpdateCollectionResult;

    fn handle(&mut self, msg: UpdateCollectionMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = CollectionRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.update_entity(&msg.collection))
    }
}

fn on_update_collection(
    (state, path, body): (State<AppState>, Path<EntityUid>, Json<CollectionEntity>),
) -> FutureResponse<HttpResponse> {
    let msg = UpdateCollectionMessage {
        collection: body.into_inner(),
    };
    // TODO: Handle UID mismatch
    let uid = path.into_inner();
    assert!(uid == *msg.collection.header().uid());
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(move |res| match res {
            Ok((_, Some(next_revision))) => {
                let next_header = EntityHeader::new(uid, next_revision);
                Ok(HttpResponse::Ok().json(next_header))
            }
            Ok((_, None)) => Err(actix_web::error::ErrorBadRequest(format_err!(
                "Inexistent entity or revision conflict"
            ))),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct DeleteCollectionMessage {
    pub uid: EntityUid,
}

pub type DeleteCollectionResult = CollectionsResult<Option<()>>;

impl Message for DeleteCollectionMessage {
    type Result = DeleteCollectionResult;
}

impl Handler<DeleteCollectionMessage> for SqliteExecutor {
    type Result = DeleteCollectionResult;

    fn handle(&mut self, msg: DeleteCollectionMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = CollectionRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.delete_entity(&msg.uid))
    }
}

fn on_delete_collection(
    (state, path): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = DeleteCollectionMessage {
        uid: path.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(Some(())) => Ok(HttpResponse::NoContent().into()),
            Ok(None) => Ok(HttpResponse::NotFound().into()),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct LoadCollectionMessage {
    pub uid: EntityUid,
    pub with_track_stats: bool,
}

pub type LoadCollectionResult = CollectionsResult<Option<CollectionEntityWithStats>>;

impl Message for LoadCollectionMessage {
    type Result = LoadCollectionResult;
}

impl Handler<LoadCollectionMessage> for SqliteExecutor {
    type Result = LoadCollectionResult;

    fn handle(&mut self, msg: LoadCollectionMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = CollectionRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            let entity = repository.load_entity(&msg.uid)?;
            if let Some(entity) = entity {
                let track_stats = if msg.with_track_stats {
                    let track_repo = TrackRepository::new(connection);
                    Some(track_repo.collection_stats(&msg.uid)?)
                } else {
                    None
                };
                Ok(Some(CollectionEntityWithStats {
                    entity,
                    stats: CollectionStats {
                        tracks: track_stats,
                    },
                }))
            } else {
                Ok(None)
            }
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WithTokensQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl WithTokensQueryParams {
    pub fn try_with_token(&self, with_token: &str) -> bool {
        match self.with {
            Some(ref with) => with.split(',').any(|token| token == with_token),
            None => false,
        }
    }
}

fn on_load_collection(
    (state, path_uid, query_with): (
        State<AppState>,
        Path<EntityUid>,
        Query<WithTokensQueryParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = LoadCollectionMessage {
        uid: path_uid.into_inner(),
        with_track_stats: query_with.into_inner().try_with_token("track-stats"),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(Some(collection)) => Ok(HttpResponse::Ok().json(collection)),
            Ok(None) => Ok(HttpResponse::NotFound().into()),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct ListCollectionsMessage {
    pub pagination: Pagination,
}

pub type ListCollectionsResult = CollectionsResult<Vec<CollectionEntity>>;

impl Message for ListCollectionsMessage {
    type Result = ListCollectionsResult;
}

impl Handler<ListCollectionsMessage> for SqliteExecutor {
    type Result = ListCollectionsResult;

    fn handle(&mut self, msg: ListCollectionsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = CollectionRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.list_entities(&msg.pagination))
    }
}

fn on_list_collections(
    (state, query_pagination): (State<AppState>, Query<Pagination>),
) -> FutureResponse<HttpResponse> {
    let msg = ListCollectionsMessage {
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(collections) => Ok(HttpResponse::Ok().json(collections)),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct CreateTrackMessage {
    pub track: Track,
}

pub type CreateTrackResult = TracksResult<TrackEntity>;

impl Message for CreateTrackMessage {
    type Result = CreateTrackResult;
}

impl Handler<CreateTrackMessage> for SqliteExecutor {
    type Result = CreateTrackResult;

    fn handle(&mut self, msg: CreateTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.create_entity(msg.track, SerializationFormat::JSON)
        })
    }
}

fn on_create_track((state, body): (State<AppState>, Json<Track>)) -> FutureResponse<HttpResponse> {
    let msg = CreateTrackMessage {
        track: body.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(track) => Ok(HttpResponse::Created().json(track.header())),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct UpdateTrackMessage {
    pub track: TrackEntity,
}

pub type UpdateTrackResult = TracksResult<(EntityRevision, Option<EntityRevision>)>;

impl Message for UpdateTrackMessage {
    type Result = UpdateTrackResult;
}

impl Handler<UpdateTrackMessage> for SqliteExecutor {
    type Result = UpdateTrackResult;

    fn handle(&mut self, msg: UpdateTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.update_entity(msg.track, SerializationFormat::JSON)
        })
    }
}

fn on_update_track(
    (state, path_uid, body): (State<AppState>, Path<EntityUid>, Json<TrackEntity>),
) -> FutureResponse<HttpResponse> {
    let uid = path_uid.into_inner();
    let msg = UpdateTrackMessage {
        track: body.into_inner(),
    };
    // TODO: Handle UID mismatch
    assert!(uid == *msg.track.header().uid());
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(move |res| match res {
            Ok((_, Some(next_revision))) => {
                let next_header = EntityHeader::new(uid, next_revision);
                Ok(HttpResponse::Ok().json(next_header))
            }
            Ok((_, None)) => Err(actix_web::error::ErrorBadRequest(format_err!(
                "Inexistent entity or revision conflict"
            ))),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct DeleteTrackMessage {
    pub uid: EntityUid,
}

pub type DeleteTrackResult = TracksResult<Option<()>>;

impl Message for DeleteTrackMessage {
    type Result = DeleteTrackResult;
}

impl Handler<DeleteTrackMessage> for SqliteExecutor {
    type Result = DeleteTrackResult;

    fn handle(&mut self, msg: DeleteTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.delete_entity(&msg.uid))
    }
}

fn on_delete_track(
    (state, path_uid): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = DeleteTrackMessage {
        uid: path_uid.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(Some(())) => Ok(HttpResponse::NoContent().into()),
            Ok(None) => Ok(HttpResponse::NotFound().into()),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug)]
pub struct LoadTrackMessage {
    pub uid: EntityUid,
}

pub type LoadTrackResult = TracksResult<Option<SerializedEntity>>;

impl Message for LoadTrackMessage {
    type Result = LoadTrackResult;
}

impl Handler<LoadTrackMessage> for SqliteExecutor {
    type Result = LoadTrackResult;

    fn handle(&mut self, msg: LoadTrackMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| repository.load_entity(&msg.uid))
    }
}

fn on_load_track(
    (state, path_uid): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = LoadTrackMessage {
        uid: path_uid.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(Some(serialized_track)) => {
                let mime_type: mime::Mime = serialized_track.format.into();
                Ok(HttpResponse::Ok()
                    .content_type(mime_type.to_string().as_str())
                    .body(serialized_track.blob))
            }
            Ok(None) => Ok(HttpResponse::NotFound().into()),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TracksQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<EntityUid>,
}

#[derive(Debug, Default)]
pub struct SearchTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: SearchTracksParams,
}

pub type SearchTracksResult = TracksResult<Vec<SerializedEntity>>;

impl Message for SearchTracksMessage {
    type Result = SearchTracksResult;
}

impl Handler<SearchTracksMessage> for SqliteExecutor {
    type Result = SearchTracksResult;

    fn handle(&mut self, msg: SearchTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.search_entities(msg.collection_uid.as_ref(), &msg.pagination, msg.params)
        })
    }
}

fn on_list_tracks(
    (state, query_tracks, query_pagination): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = SearchTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        pagination: query_pagination.into_inner(),
        ..Default::default()
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(serialized_tracks) => concat_serialized_entities_into_json_array(serialized_tracks),
            Err(e) => Err(e.into()),
        })
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

fn on_search_tracks(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<SearchTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = SearchTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        pagination: query_pagination.into_inner(),
        params: body.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(serialized_tracks) => concat_serialized_entities_into_json_array(serialized_tracks),
            Err(e) => Err(e.into()),
        })
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

#[derive(Debug)]
pub struct LocateTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: LocateTracksParams,
}

pub type LocateTracksResult = TracksResult<Vec<SerializedEntity>>;

impl Message for LocateTracksMessage {
    type Result = LocateTracksResult;
}

impl Handler<LocateTracksMessage> for SqliteExecutor {
    type Result = LocateTracksResult;

    fn handle(&mut self, msg: LocateTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.locate_entities(msg.collection_uid.as_ref(), &msg.pagination, msg.params)
        })
    }
}

fn on_locate_tracks(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<LocateTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = LocateTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        pagination: query_pagination.into_inner(),
        params: body.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(serialized_tracks) => concat_serialized_entities_into_json_array(serialized_tracks),
            Err(e) => Err(e.into()),
        })
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

#[derive(Debug)]
pub struct ReplaceTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub params: ReplaceTracksParams,
    pub format: SerializationFormat,
}

pub type ReplaceTracksResult = TracksResult<ReplacedTracks>;

impl Message for ReplaceTracksMessage {
    type Result = ReplaceTracksResult;
}

impl Handler<ReplaceTracksMessage> for SqliteExecutor {
    type Result = ReplaceTracksResult;

    fn handle(&mut self, msg: ReplaceTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.replace_entities(msg.collection_uid.as_ref(), msg.params, msg.format)
        })
    }
}

fn on_replace_tracks(
    (state, query_tracks, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Json<ReplaceTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = ReplaceTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        format: SerializationFormat::JSON,
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(res) => Ok(HttpResponse::Ok().json(res)),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug, Default)]
struct ListTracksFieldsMessage {
    pub collection_uid: Option<EntityUid>,
    pub with_fields: Vec<StringField>,
    pub pagination: Pagination,
}

pub type ListTracksFieldsResult = TracksResult<Vec<StringFieldCounts>>;

impl Message for ListTracksFieldsMessage {
    type Result = ListTracksFieldsResult;
}

impl Handler<ListTracksFieldsMessage> for SqliteExecutor {
    type Result = ListTracksFieldsResult;

    fn handle(&mut self, msg: ListTracksFieldsMessage, _: &mut Self::Context) -> Self::Result {
        let mut results: Vec<StringFieldCounts> = Vec::with_capacity(msg.with_fields.len());
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            for field in msg.with_fields.into_iter() {
                let result =
                    repository.list_fields(msg.collection_uid.as_ref(), field, &msg.pagination)?;
                results.push(result);
            }
            Ok(results)
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TracksWithStringFieldsQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl TracksWithStringFieldsQueryParams {
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
}

fn on_list_tracks_fields(
    (state, query_tracks, query_with, query_pagination): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<TracksWithStringFieldsQueryParams>,
        Query<Pagination>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = ListTracksFieldsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        with_fields: query_with.into_inner().with_fields(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(tags) => Ok(HttpResponse::Ok().json(tags)),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TracksWithTagFacetsQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl TracksWithTagFacetsQueryParams {
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
}

#[derive(Debug, Default)]
struct ListTracksTagsMessage {
    pub collection_uid: Option<EntityUid>,
    pub query_params: TracksWithTagFacetsQueryParams,
    pub pagination: Pagination,
}

pub type ListTracksTagsResult = TracksResult<Vec<ScoredTagCount>>;

impl Message for ListTracksTagsMessage {
    type Result = ListTracksTagsResult;
}

impl Handler<ListTracksTagsMessage> for SqliteExecutor {
    type Result = ListTracksTagsResult;

    fn handle(&mut self, msg: ListTracksTagsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.list_tags(
                msg.collection_uid.as_ref(),
                msg.query_params.with_facets().as_ref(),
                &msg.pagination,
            )
        })
    }
}

fn on_list_tracks_tags(
    (state, query_tracks, query_with, query_pagination): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<TracksWithTagFacetsQueryParams>,
        Query<Pagination>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = ListTracksTagsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        query_params: query_with.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(tags) => Ok(HttpResponse::Ok().json(tags)),
            Err(e) => Err(e.into()),
        })
        .responder()
}

#[derive(Debug, Default)]
struct ListTracksTagsFacetsMessage {
    pub collection_uid: Option<EntityUid>,
    pub query_params: TracksWithTagFacetsQueryParams,
    pub pagination: Pagination,
}

pub type ListTracksTagsFacetsResult = TracksResult<Vec<TagFacetCount>>;

impl Message for ListTracksTagsFacetsMessage {
    type Result = ListTracksTagsFacetsResult;
}

impl Handler<ListTracksTagsFacetsMessage> for SqliteExecutor {
    type Result = ListTracksTagsFacetsResult;

    fn handle(&mut self, msg: ListTracksTagsFacetsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.list_tag_facets(
                msg.collection_uid.as_ref(),
                msg.query_params.with_facets().as_ref(),
                &msg.pagination,
            )
        })
    }
}

fn on_list_tracks_tags_facets(
    (state, query_tracks, query_with, query_pagination): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<TracksWithTagFacetsQueryParams>,
        Query<Pagination>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = ListTracksTagsFacetsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        query_params: query_with.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .from_err()
        .and_then(|res| match res {
            Ok(tags) => Ok(HttpResponse::Ok().json(tags)),
            Err(e) => Err(e.into()),
        })
        .responder()
}

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

const DB_URL_ARG: &str = "DB_URL";
const LISTEN_ADDR_ARG: &str = "LISTEN_ADDR";
const SKIP_DATABASE_MAINTENANCE_ARG: &str = "SKIP_DATABASE_MAINTENANCE";
const VERBOSITY_ARG: &str = "VERBOSITY";

pub fn main() -> Result<(), Error> {
    let matches = App::new("aoide")
            .version("0.0.1")
            //.author("")
            //.about("")
            .arg(Arg::with_name(DB_URL_ARG)
                .help("Sets the database URL")
                .default_value(":memory:")
                .index(1))
            .arg(Arg::with_name(LISTEN_ADDR_ARG)
                .short("l")
                .long("listen")
                .default_value("localhost:8080")
                .help("Sets the network listen address"))
            .arg(Arg::with_name(SKIP_DATABASE_MAINTENANCE_ARG)
                .long("skipDatabaseMaintenance")
                .help("Skips database schema migration and maintenance tasks on startup"))
            .arg(Arg::with_name(VERBOSITY_ARG)
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity (= number of occurrences)"))
            .get_matches();

    let verbosity = matches.occurrences_of(VERBOSITY_ARG);
    init_env_logger_verbosity(verbosity.min(8) as u8);

    let db_url = matches.value_of(DB_URL_ARG).unwrap();
    info!("Database URL: {}", db_url);

    let listen_addr = matches.value_of(LISTEN_ADDR_ARG).unwrap();
    info!("Network listen address: {}", listen_addr);

    // Workaround: Use a pool of size 1 to avoid 'database is locked'
    // errors due to multi-threading.
    let connection_pool =
        create_connection_pool(db_url, 1).expect("Failed to create database connection pool");

    if matches.is_present(SKIP_DATABASE_MAINTENANCE_ARG) {
        info!("Skipping database maintenance");
    } else {
        migrate_database_schema(&connection_pool).unwrap();
        cleanup_database_storage(&connection_pool).unwrap();
        repair_database_storage(&connection_pool).unwrap();
    }

    info!("Creating actor system");
    let sys = actix::System::new("aoide");
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

    info!("Listening for requests at http://{}", listen_addr);
    let _ = sys.run();

    info!("Exiting");
    Ok(())
}
