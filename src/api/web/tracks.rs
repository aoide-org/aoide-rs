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

use aoide_storage::{
    api::{
        serde::{SerializationFormat, SerializedEntity},
        track::{TrackAlbums, TrackTags, Tracks, TracksResult},
        CountAlbumTracksParams, CountTagsParams, FacetCount, LocateTracksParams, Pagination,
        ReplaceTracksParams, ReplacedTracks, SearchTracksParams, TagCount,
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
        .map_err(Error::compat)
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
        .map_err(Error::compat)
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
        .map_err(Error::compat)
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
        .map_err(Error::compat)
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
        .map_err(Error::compat)
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
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
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
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
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
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default)]
struct CountAlbumTracksMessage {
    pub collection_uid: Option<EntityUid>,
    pub params: CountAlbumTracksParams,
    pub pagination: Pagination,
}

pub type CountAlbumTracksResult = TracksResult<Vec<AlbumTracksCount>>;

impl Message for CountAlbumTracksMessage {
    type Result = CountAlbumTracksResult;
}

impl Handler<CountAlbumTracksMessage> for SqliteExecutor {
    type Result = CountAlbumTracksResult;

    fn handle(&mut self, msg: CountAlbumTracksMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.count_albums(msg.collection_uid.as_ref(), &msg.params, msg.pagination)
        })
    }
}

pub fn on_count_track_albums(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<CountAlbumTracksParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = CountAlbumTracksMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default)]
struct CountTrackTagsMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: CountTagsParams,
}

pub type CountTrackTagsResult = TracksResult<Vec<TagCount>>;

impl Message for CountTrackTagsMessage {
    type Result = CountTrackTagsResult;
}

impl Handler<CountTrackTagsMessage> for SqliteExecutor {
    type Result = CountTrackTagsResult;

    fn handle(&mut self, msg: CountTrackTagsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        let collection_uid = msg.collection_uid;
        let pagination = msg.pagination;
        let facets = msg.params.facets.map(|mut facets| {
            facets.sort();
            facets.dedup();
            facets
        });
        let facets = facets.as_ref().map(|facets| {
            facets
                .iter()
                .map(AsRef::as_ref)
                .map(String::as_str)
                .collect()
        });
        connection.transaction::<_, Error, _>(|| {
            repository.count_tags(
                collection_uid.as_ref(),
                facets.as_ref().map(Vec::as_slice),
                pagination,
            )
        })
    }
}

pub fn on_count_track_tags(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<CountTagsParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = CountTrackTagsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}

#[derive(Debug, Default)]
struct CountTrackFacetsMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
    pub params: CountTagsParams,
}

pub type CountTrackFacetsResult = TracksResult<Vec<FacetCount>>;

impl Message for CountTrackFacetsMessage {
    type Result = CountTrackFacetsResult;
}

impl Handler<CountTrackFacetsMessage> for SqliteExecutor {
    type Result = CountTrackFacetsResult;

    fn handle(&mut self, msg: CountTrackFacetsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        let collection_uid = msg.collection_uid;
        let pagination = msg.pagination;
        let facets = msg.params.facets.map(|mut facets| {
            facets.sort();
            facets.dedup();
            facets
        });
        let facets = facets.as_ref().map(|facets| {
            facets
                .iter()
                .map(AsRef::as_ref)
                .map(String::as_str)
                .collect()
        });
        connection.transaction::<_, Error, _>(|| {
            repository.count_facets(
                collection_uid.as_ref(),
                facets.as_ref().map(Vec::as_slice),
                pagination,
            )
        })
    }
}

pub fn on_count_track_facets(
    (state, query_tracks, query_pagination, body): (
        State<AppState>,
        Query<TracksQueryParams>,
        Query<Pagination>,
        Json<CountTagsParams>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = CountTrackFacetsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        params: body.into_inner(),
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .flatten()
        .map_err(Error::compat)
        .from_err()
        .and_then(|res| Ok(HttpResponse::Ok().json(res)))
        .responder()
}
