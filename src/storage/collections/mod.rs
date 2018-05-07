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

mod models;

use self::models::*;

mod schema;

use self::schema::*;

use std::i64;

use diesel::prelude::*;
use diesel;

use aoide_core::domain::entity::EntityUid;
use aoide_core::domain::collection::*;

use storage::*;

use usecases::*;
use usecases::result::Pagination;

///////////////////////////////////////////////////////////////////////
/// CollectionRepository
///////////////////////////////////////////////////////////////////////

pub struct CollectionRepository<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> CollectionRepository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }
}

type IdColumn = (
    collections_entity::id,
);

const ID_COLUMN: IdColumn = (
    collections_entity::id,
);

impl<'a> EntityStorage for CollectionRepository<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        let target = collections_entity::table
            .select(ID_COLUMN)
            .filter(collections_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableStorageId>(self.connection)
            .optional()?;
        Ok(result.map(|r| r.id))
    }
}

impl<'a> Collections for CollectionRepository<'a> {
    fn create_entity(&self, body: CollectionBody) -> CollectionsResult<CollectionEntity> {
        let entity = CollectionEntity::with_body(body);
        {
            let insertable = InsertableCollectionsEntity::bind(&entity);
            let query = diesel::insert_into(collections_entity::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(entity)
    }

    fn update_entity(&self, entity: &CollectionEntity) -> CollectionsResult<Option<EntityRevision>> {
        let next_revision = entity.header().revision().next();
        {
            let updatable = UpdatableCollectionsEntity::bind(&next_revision, &entity.body());
            let target = collections_entity::table
                .filter(collections_entity::uid.eq(entity.header().uid().as_str())
                    .and(collections_entity::rev_ordinal.eq(entity.header().revision().ordinal() as i64))
                    .and(collections_entity::rev_timestamp.eq(entity.header().revision().timestamp().naive_utc())));
            let query = diesel::update(target).set(&updatable);
            let rows_affected: usize = query.execute(self.connection)?;
            assert!(rows_affected <= 1);
            if rows_affected <= 0 {
                return Ok(None);
            }
        }
        Ok(Some(next_revision))
    }

    fn remove_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<()>> {
        let target = collections_entity::table.filter(collections_entity::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.connection)?;
        assert!(rows_affected <= 1);
        if rows_affected <= 0 {
            return Ok(None);
        }
        Ok(Some(()))
    }

    fn find_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>> {
        let target = collections_entity::table.filter(collections_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableCollectionsEntity>(self.connection)
            .optional()?;
        Ok(result.map(|r| r.into()))
    }
    
    fn find_recently_revisioned_entities(&self, pagination: &Pagination) -> CollectionsResult<Vec<CollectionEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let target = collections_entity::table
            .order(collections_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = target.load::<QueryableCollectionsEntity>(self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>> {
        let target = collections_entity::table.filter(collections_entity::name.eq(name));
        let results = target.load::<QueryableCollectionsEntity>(self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name_starting_with(
        &self,
        name_prefix: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let target = collections_entity::table
            .filter(collections_entity::name.like(format!("{}%", name_prefix)))
            .order(collections_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = target.load::<QueryableCollectionsEntity>(self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name_containing(
        &self,
        partial_name: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let target = collections_entity::table
            .filter(collections_entity::name.like(format!("%{}%", partial_name)))
            .order(collections_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = target.load::<QueryableCollectionsEntity>(self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    embed_migrations!("db/migrations/sqlite");

    fn establish_connection() -> SqliteConnection {
        let connection =
            SqliteConnection::establish(":memory:").expect("in-memory database connection");
        embedded_migrations::run(&connection).expect("database schema migration");
        connection
    }

    #[test]
    fn create_entity() {
        let connection = establish_connection();
        let repository = CollectionRepository::new(&connection);
        let entity = repository
            .create_entity(CollectionBody {
                name: "Test Collection".into(),
                description: Some("Description".into()),
            })
            .unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
    }

    #[test]
    fn update_entity() {
        let connection = establish_connection();
        let repository = CollectionRepository::new(&connection);
        let mut entity = repository
            .create_entity(CollectionBody {
                name: "Test Collection".into(),
                description: Some("Description".into()),
            })
            .unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        let prev_revision = entity.header().revision();
        entity.body_mut().name = "Renamed Collection".into();
        let next_revision = repository.update_entity(&entity).unwrap().unwrap();
        println!("Updated entity: {:?}", entity);
        assert!(prev_revision < next_revision);
        assert!(entity.header().revision() == prev_revision);
        entity.update_revision(next_revision);
        assert!(entity.header().revision() == next_revision);
    }

    #[test]
    fn remove_entity() {
        let connection = establish_connection();
        let repository = CollectionRepository::new(&connection);
        let entity = repository
            .create_entity(CollectionBody {
                name: "Test Collection".into(),
                description: None,
            })
            .unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        assert!(Some(()) == repository.remove_entity(&entity.header().uid()).unwrap());
        println!("Removed entity: {}", entity.header().uid());
    }
}
