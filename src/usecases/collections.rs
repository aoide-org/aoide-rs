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

use aoide_core::{
    collection::{Collection, Entity},
    entity::{EntityHeader, EntityRevision, EntityUid},
};

use aoide_repo::{
    collection::{Repo as _, TrackStats},
    track::Repo as _,
    Pagination, RepoResult,
};

use aoide_repo_sqlite::{collection::Repository, track::Repository as TrackRepository};

///////////////////////////////////////////////////////////////////////

pub fn create_collection(
    db: &SqlitePooledConnection,
    new_collection: Collection,
) -> RepoResult<EntityHeader> {
    let repository = Repository::new(&*db);
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr, new_collection);
    db.transaction::<_, Error, _>(|| repository.insert_collection(&entity).map(|()| entity.hdr))
}

pub fn update_collection(
    db: &SqlitePooledConnection,
    entity: &Entity,
) -> RepoResult<(EntityRevision, Option<EntityRevision>)> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.update_collection(entity))
}

pub fn delete_collection(db: &SqlitePooledConnection, uid: &EntityUid) -> RepoResult<Option<()>> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| repository.delete_collection(uid))
}

pub fn load_collection(
    db: &SqlitePooledConnection,
    uid: &EntityUid,
    with_track_stats: bool,
) -> RepoResult<Option<(Entity, Option<TrackStats>)>> {
    let repository = Repository::new(&*db);
    db.transaction::<_, Error, _>(|| {
        let entity = repository.load_collection(uid)?;
        if let Some(entity) = entity {
            let track_stats = if with_track_stats {
                let track_repo = TrackRepository::new(&*db);
                Some(track_repo.collect_collection_track_stats(uid)?)
            } else {
                None
            };
            Ok(Some((entity, track_stats)))
        } else {
            Ok(None)
        }
    })
}

pub fn list_collections(
    pooled_connection: &SqlitePooledConnection,
    pagination: Pagination,
) -> RepoResult<Vec<Entity>> {
    let repository = Repository::new(&*pooled_connection);
    pooled_connection.transaction::<_, Error, _>(|| repository.list_collections(pagination))
}
