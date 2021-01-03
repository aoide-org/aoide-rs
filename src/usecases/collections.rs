// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_repo_sqlite::Connection as DbConnection;

///////////////////////////////////////////////////////////////////////

pub fn create_collection(
    db: &SqlitePooledConnection,
    new_collection: Collection,
) -> RepoResult<EntityHeader> {
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr, new_collection);
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.insert_collection(&entity).map(|()| entity.hdr)
    })
}

pub fn update_collection(
    db: &SqlitePooledConnection,
    entity: &Entity,
) -> RepoResult<(EntityRevision, Option<EntityRevision>)> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.update_collection(entity)
    })
}

pub fn delete_collection(db: &SqlitePooledConnection, uid: &EntityUid) -> RepoResult<Option<()>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.delete_collection(uid)
    })
}

pub fn load_collection(
    db: &SqlitePooledConnection,
    uid: &EntityUid,
    with_track_stats: bool,
) -> RepoResult<Option<(Entity, Option<TrackStats>)>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        let entity = repo.load_collection(uid)?;
        if let Some(entity) = entity {
            let track_stats = if with_track_stats {
                Some(repo.collect_collection_track_stats(uid)?)
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
    db: &SqlitePooledConnection,
    pagination: Pagination,
) -> RepoResult<Vec<Entity>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.list_collections(pagination)
    })
}
