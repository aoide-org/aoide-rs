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

mod schema;

use std::i64;

use self::schema::collection_entity;
use self::schema::active_collection;

use chrono::{DateTime, Utc};
use chrono::naive::NaiveDateTime;

use diesel::prelude::*;
use diesel;

use log;

use aoide_core::domain::entity::*;
use aoide_core::domain::collection::*;

use storage::*;

use usecases::*;

///////////////////////////////////////////////////////////////////////
/// CollectionRecord
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "collection_entity"]
pub struct InsertableCollectionEntity<'a> {
    pub uid: &'a str,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> InsertableCollectionEntity<'a> {
    pub fn borrow(entity: &'a CollectionEntity) -> Self {
        Self {
            uid: entity.header().uid().as_str(),
            rev_ordinal: entity.header().revision().ordinal() as i64,
            rev_timestamp: entity.header().revision().timestamp().naive_utc(),
            name: &entity.body().name,
            description: entity.body().description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "collection_entity"]
pub struct UpdatableCollectionEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> UpdatableCollectionEntity<'a> {
    pub fn borrow(next_revision: &EntityRevision, body: &'a CollectionBody) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_timestamp: next_revision.timestamp().naive_utc(),
            name: &body.name,
            description: body.description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, Queryable)]
pub struct QueryableCollectionEntity {
    pub id: i64,
    pub uid: String,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: String,
    pub description: Option<String>,
}

impl From<QueryableCollectionEntity> for CollectionEntity {
    fn from(from: QueryableCollectionEntity) -> Self {
        let uid: EntityUid = from.uid.into();
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let body = CollectionBody {
            name: from.name,
            description: from.description,
        };
        Self::new(header, body)
    }
}

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
    collection_entity::id,
);

const ID_COLUMN: IdColumn = (
    collection_entity::id,
);

impl<'a> EntityStorage for CollectionRepository<'a> {
    fn lookup_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        let target = collection_entity::table
            .select(ID_COLUMN)
            .filter(collection_entity::uid.eq(uid.as_str()));
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
            let insertable = InsertableCollectionEntity::borrow(&entity);
            let query = diesel::insert_into(collection_entity::table).values(&insertable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            query.execute(self.connection)?;
        }
        if log_enabled!(log::Level::Debug) {
            debug!("Created collection entity: {:?}", entity.header());
        }
        Ok(entity)
    }

    fn update_entity(&self, entity: &CollectionEntity) -> CollectionsResult<Option<EntityRevision>> {
        let next_revision = entity.header().revision().next();
        {
            let updatable = UpdatableCollectionEntity::borrow(&next_revision, &entity.body());
            let target = collection_entity::table
                .filter(collection_entity::uid.eq(entity.header().uid().as_str())
                    .and(collection_entity::rev_ordinal.eq(entity.header().revision().ordinal() as i64))
                    .and(collection_entity::rev_timestamp.eq(entity.header().revision().timestamp().naive_utc())));
            let query = diesel::update(target).set(&updatable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            let rows_affected: usize = query.execute(self.connection)?;
            assert!(rows_affected <= 1);
            if rows_affected <= 0 {
                return Ok(None);
            }
        }
        if log_enabled!(log::Level::Debug) {
            debug!("Updated collection entity: {:?} -> {:?}", entity.header(), next_revision);
        }
        Ok(Some(next_revision))
    }

    fn remove_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<()>> {
        let target = collection_entity::table.filter(collection_entity::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Executing SQLite query: {}",
                diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
            );
        }
        let rows_affected: usize = query.execute(self.connection)?;
        assert!(rows_affected <= 1);
        if rows_affected <= 0 {
            return Ok(None);
        }
        if log_enabled!(log::Level::Debug) {
            debug!("Removed collection entity: {}", uid);
        }
        Ok(Some(()))
    }

    fn find_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>> {
        let target = collection_entity::table.filter(collection_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableCollectionEntity>(self.connection)
            .optional()?;
        if log_enabled!(log::Level::Debug) {
            match &result {
                &None => {
                    debug!("Found no collection entity with uid '{}'", uid);
                }
                &Some(_) => {
                    debug!("Found a collection entity with uid '{}'", uid);
                }
            }
        }
        Ok(result.map(|r| r.into()))
    }
    
    fn find_all_entities(&self, pagination: &Pagination) -> CollectionsResult<Vec<CollectionEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let target = collection_entity::table
            .order(collection_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Loaded {} collection entities",
                results.len(),
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>> {
        let target = collection_entity::table.filter(collection_entity::name.eq(name));
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Loaded {} collection entities by name '{}'",
                results.len(),
                name
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name_starting_with(
        &self,
        name_prefix: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let target = collection_entity::table
            .filter(collection_entity::name.like(format!("{}%", name_prefix)))
            .order(collection_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Loaded {} collection entities by name starting with '{}'",
                results.len(),
                name_prefix
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name_containing(
        &self,
        partial_name: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination.limit.map(|limit| limit as i64).unwrap_or(i64::MAX);
        let target = collection_entity::table
            .filter(collection_entity::name.like(format!("%{}%", partial_name)))
            .order(collection_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Loaded {} collection entities by name containing '{}'",
                results.len(),
                partial_name
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn activate_collection(&self, _uid: &EntityUid) -> CollectionsResult<()> {
        bail!("TODO: Implement activation of a collection");
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
