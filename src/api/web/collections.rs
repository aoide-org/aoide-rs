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
        collection::{CollectionEntityWithStats, CollectionStats, Collections, CollectionsResult},
        track::Tracks,
        Pagination,
    },
    storage::{collection::CollectionRepository, track::TrackRepository},
};

///////////////////////////////////////////////////////////////////////

pub struct CollectionsHandler {
    db: SqlitePooledConnection,
}

impl CollectionsHandler {
    pub fn new(db: SqlitePooledConnection) -> Self {
        Self { db }
    }

    pub fn handle_create(
        &self,
        new_collection: Collection,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        create_collection(&self.db, new_collection)
            .map_err(warp::reject::custom)
            .map(|val| {
                warp::reply::with_status(warp::reply::json(&val), warp::http::StatusCode::CREATED)
            })
    }

    pub fn handle_update(
        &self,
        uid: EntityUid,
        collection: CollectionEntity,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        if uid != *collection.header().uid() {
            return Err(warp::reject::custom(failure::format_err!(
                "Mismatching UIDs: {} <> {}",
                uid,
                collection.header().uid(),
            )));
        }
        update_collection(&self.db, &collection)
            .and_then(move |res| match res {
                (_, Some(next_revision)) => {
                    let next_header = aoide_domain::entity::EntityHeader::new(uid, next_revision);
                    Ok(warp::reply::json(&next_header))
                }
                (_, None) => Err(failure::format_err!(
                    "Inexistent entity or revision conflict"
                )),
            })
            .map_err(warp::reject::custom)
    }

    pub fn handle_delete(
        &self,
        uid: EntityUid,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        delete_collection(&self.db, &uid)
            .map_err(warp::reject::custom)
            .map(|res| {
                warp::reply::with_status(
                    warp::reply(),
                    res.map(|()| warp::http::StatusCode::NO_CONTENT)
                        .unwrap_or(warp::http::StatusCode::NOT_FOUND),
                )
            })
    }

    pub fn handle_load(
        &self,
        uid: EntityUid,
        params: WithTokensQueryParams,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        load_collection(&self.db, &uid, params.try_with_token("track-stats"))
            .map_err(warp::reject::custom)
            .and_then(|res| match res {
                Some(val) => Ok(warp::reply::json(&val)),
                None => Err(warp::reject::not_found()),
            })
    }

    pub fn handle_list(
        &self,
        pagination: Pagination,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        list_collections(&self.db, pagination)
            .map_err(warp::reject::custom)
            .map(|val| warp::reply::json(&val))
    }
}

fn create_collection(
    db: &SqlitePooledConnection,
    new_collection: Collection,
) -> CollectionsResult<CollectionEntity> {
    let repository = CollectionRepository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.create_entity(new_collection))
}

fn update_collection(
    db: &SqlitePooledConnection,
    collection: &CollectionEntity,
) -> CollectionsResult<(EntityRevision, Option<EntityRevision>)> {
    let repository = CollectionRepository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.update_entity(collection))
}

fn delete_collection(
    db: &SqlitePooledConnection,
    uid: &EntityUid,
) -> CollectionsResult<Option<()>> {
    let repository = CollectionRepository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.delete_entity(uid))
}

fn load_collection(
    db: &SqlitePooledConnection,
    uid: &EntityUid,
    with_track_stats: bool,
) -> CollectionsResult<Option<CollectionEntityWithStats>> {
    let repository = CollectionRepository::new(&*db);
    db.transaction::<_, Error, _>(|| {
        let entity = repository.load_entity(uid)?;
        if let Some(entity) = entity {
            let track_stats = if with_track_stats {
                let track_repo = TrackRepository::new(&*db);
                Some(track_repo.collection_stats(uid)?)
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

pub fn list_collections(
    pooled_connection: &SqlitePooledConnection,
    pagination: Pagination,
) -> CollectionsResult<Vec<CollectionEntity>> {
    let repository = CollectionRepository::new(&*pooled_connection);
    pooled_connection.transaction::<_, Error, _>(|| repository.list_entities(pagination))
}
