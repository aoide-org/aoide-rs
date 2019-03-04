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

use super::*;

use crate::core::{entity::*, track::*};

use aoide_storage::{
    api::{
        serde::{SerializationFormat, SerializedEntity},
        track::{TrackTags, Tracks, TracksResult},
        LocateTracksParams, Pagination, ReplaceTracksParams, ReplacedTracks, ScoredTagCount,
        SearchTracksParams, StringField, StringFieldCounts, TagFacetCount,
    },
    storage::track::TrackRepository,
};

use actix_web::AsyncResponder;

use futures::future::Future;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracksQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<EntityUid>,
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

pub fn on_create_track(
    (state, body): (State<AppState>, Json<Track>),
) -> FutureResponse<HttpResponse> {
    let msg = CreateTrackMessage {
        track: body.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Created().json(res.header())))
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

pub fn on_update_track(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(move |res| match res {
            (_, Some(next_revision)) => {
                let next_header = EntityHeader::new(uid, next_revision);
                Ok(HttpResponse::Ok().json(next_header))
            }
            (_, None) => Err(actix_web::error::ErrorBadRequest(failure::format_err!(
                "Inexistent entity or revision conflict"
            ))),
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

pub fn on_delete_track(
    (state, path_uid): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = DeleteTrackMessage {
        uid: path_uid.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| match res {
            Some(_) => Ok(HttpResponse::NoContent().into()),
            None => Ok(HttpResponse::NotFound().into()),
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

pub fn on_load_track(
    (state, path_uid): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = LoadTrackMessage {
        uid: path_uid.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| match res {
            Some(serialized_track) => {
                let mime_type: mime::Mime = serialized_track.format.into();
                Ok(HttpResponse::Ok()
                    .content_type(mime_type.to_string().as_str())
                    .body(serialized_track.blob))
            }
            None => Ok(HttpResponse::NotFound().into()),
        })
        .responder()
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
            repository.search_entities(msg.collection_uid.as_ref(), msg.pagination, msg.params)
        })
    }
}

pub fn on_list_tracks(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|serialized_tracks| SerializedEntity::slice_to_json_array(&serialized_tracks))
        .from_err()
        .and_then(|json| {
            Ok(HttpResponse::Ok()
                .content_type(mime::APPLICATION_JSON.to_string().as_str())
                .body(json))
        })
        .responder()
}

pub fn on_search_tracks(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|serialized_tracks| SerializedEntity::slice_to_json_array(&serialized_tracks))
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
            repository.locate_entities(msg.collection_uid.as_ref(), msg.pagination, msg.params)
        })
    }
}

pub fn on_locate_tracks(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|serialized_tracks| SerializedEntity::slice_to_json_array(&serialized_tracks))
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

pub fn on_replace_tracks(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
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
            for field in msg.with_fields {
                let result =
                    repository.list_fields(msg.collection_uid.as_ref(), field, msg.pagination)?;
                results.push(result);
            }
            Ok(results)
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracksWithStringFieldsQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    with: Option<String>,
}

impl TracksWithStringFieldsQueryParams {
    pub fn with_fields(&self) -> Vec<StringField> {
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
                log::warn!(
                    "{} unrecognized field selector(s) in '{}'",
                    unrecognized_field_count,
                    field_list
                );
            }
            result.sort();
            result.dedup();
        }
        result
    }
}

pub fn on_list_tracks_fields(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TracksWithTagFacetsQueryParams {
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
                msg.pagination,
            )
        })
    }
}

pub fn on_list_tracks_tags(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
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
                msg.pagination,
            )
        })
    }
}

pub fn on_list_tracks_tags_facets(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}
