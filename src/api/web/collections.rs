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

use actix::prelude::*;

use actix_web::{error, *};

use diesel::prelude::*;

use failure::Error;

use futures::future::Future;

use aoide_core::domain::{collection::*, entity::*};

use aoide_storage::{
    api::{
        collection::{CollectionEntityWithStats, CollectionStats, Collections, CollectionsResult},
        track::Tracks,
        Pagination,
    },
    storage::{collection::CollectionRepository, track::TrackRepository},
};

use super::{AppState, SqliteExecutor, WithTokensQueryParams};

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

pub fn on_create_collection(
    (state, body): (State<AppState>, Json<Collection>),
) -> FutureResponse<HttpResponse> {
    let msg = CreateCollectionMessage {
        collection: body.into_inner(),
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

pub fn on_update_collection(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(move |res| match res {
            (_, Some(next_revision)) => {
                let next_header = EntityHeader::new(uid, next_revision);
                Ok(HttpResponse::Ok().json(next_header))
            }
            (_, None) => Err(error::ErrorBadRequest(format_err!(
                "Inexistent entity or revision conflict"
            ))),
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

pub fn on_delete_collection(
    (state, path): (State<AppState>, Path<EntityUid>),
) -> FutureResponse<HttpResponse> {
    let msg = DeleteCollectionMessage {
        uid: path.into_inner(),
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

pub fn on_load_collection(
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
        .flatten()
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| match res {
            Some(collection) => Ok(HttpResponse::Ok().json(collection)),
            None => Ok(HttpResponse::NotFound().into()),
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
        connection.transaction::<_, Error, _>(|| repository.list_entities(msg.pagination))
    }
}

pub fn on_list_collections(
    (state, query_pagination): (State<AppState>, Query<Pagination>),
) -> FutureResponse<HttpResponse> {
    let msg = ListCollectionsMessage {
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
