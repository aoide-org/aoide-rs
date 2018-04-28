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
use self::schema::collection_entity;
use self::schema::active_collection;

use chrono::{DateTime, Utc};
use chrono::naive::NaiveDateTime;

use diesel::prelude::*;
use diesel;

use log;

use aoide_core::domain::entity::*;
use aoide_core::domain::collection::*;
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
}

impl<'a> InsertableCollectionEntity<'a> {
    pub fn from_entity(entity: &'a CollectionEntity) -> Self {
        Self {
            uid: entity.header().uid().as_str(),
            rev_ordinal: entity.header().revision().ordinal() as i64,
            rev_timestamp: entity.header().revision().timestamp().naive_utc(),
            name: &entity.body().name,
        }
    }
}

#[derive(Debug, AsChangeset)]
#[table_name = "collection_entity"]
pub struct UpdatableCollectionEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
}

impl<'a> UpdatableCollectionEntity<'a> {
    pub fn from_entity_revision(entity: &'a CollectionEntity, revision: EntityRevision) -> Self {
        Self {
            rev_ordinal: revision.ordinal() as i64,
            rev_timestamp: revision.timestamp().naive_utc(),
            name: &entity.body().name,
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
}

impl From<QueryableCollectionEntity> for CollectionEntity {
    fn from(from: QueryableCollectionEntity) -> Self {
        let uid: EntityUid = from.uid.into();
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let body = CollectionBody { name: from.name };
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

impl From<diesel::result::Error> for CollectionsError {
    fn from(from: diesel::result::Error) -> Self {
        match from {
            diesel::result::Error::NotFound => CollectionsError::NotFound,
            _ => {
                error!("Unexpected database error: {}", from);
                CollectionsError::Unexpected
            }
        }
    }
}

impl<'a> Collections for CollectionRepository<'a> {
    fn create_entity(&self, body: CollectionBody) -> CollectionsResult<CollectionEntity> {
        let entity = CollectionEntity::with_body(body);
        {
            let insertable = InsertableCollectionEntity::from_entity(&entity);
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

    fn update_entity(&self, entity: &mut CollectionEntity) -> CollectionsResult<EntityRevision> {
        let next_revision = entity.header().revision().next();
        {
            let updatable = UpdatableCollectionEntity::from_entity_revision(&entity, next_revision);
            let target = collection_entity::table
                .filter(collection_entity::uid.eq(entity.header().uid().as_str()));
            let query = diesel::update(target).set(&updatable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            query.execute(self.connection)?;
        }
        entity.update_revision(next_revision);
        if log_enabled!(log::Level::Debug) {
            debug!("Updated collection entity: {:?}", entity.header());
        }
        Ok(next_revision)
    }

    fn remove_entity(&self, uid: &EntityUid) -> CollectionsResult<()> {
        let target = collection_entity::table.filter(collection_entity::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Executing SQLite query: {}",
                diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
            );
        }
        query.execute(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!("Removed collection entity: {}", uid);
        }
        Ok(())
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

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>> {
        let target = collection_entity::table.filter(collection_entity::name.eq(name));
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Found {} collection entities by name '{}'",
                results.len(),
                name
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name_starting_with(
        &self,
        name_prefix: &str,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let target = collection_entity::table
            .filter(collection_entity::name.like(format!("{}%", name_prefix)));
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Found {} collection entities by name starting with '{}'",
                results.len(),
                name_prefix
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn find_entities_by_name_containing(
        &self,
        partial_name: &str,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let target = collection_entity::table
            .filter(collection_entity::name.like(format!("%{}%", partial_name)));
        let results = target.load::<QueryableCollectionEntity>(self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Found {} collection entities by name containing '{}'",
                results.len(),
                partial_name
            );
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn activate_collection(&self, _uid: &EntityUid) -> CollectionsResult<()> {
        error!("TODO: Implement activation of a collection");
        Err(CollectionsError::Unexpected)
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
        let connection = SqliteConnection::establish(":memory:").expect("in-memory database connection");
        embedded_migrations::run(&connection).expect("database schema migration");
        connection
    }

    #[test]
    fn create_entity() {
        let connection = establish_connection();
        let repository = CollectionRepository::new(&connection);
        let entity = repository.create_entity(CollectionBody { name: "Test Collection".into() }).unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
    }

    #[test]
    fn update_entity() {
        let connection = establish_connection();
        let repository = CollectionRepository::new(&connection);
        let mut entity = repository.create_entity(CollectionBody { name: "Test Collection".into() }).unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        let initial_revision = entity.header().revision();
        entity.body_mut().name = "Renamed Collection".into();
        let updated_revision = repository.update_entity(&mut entity).unwrap();
        println!("Updated entity: {:?}", entity);
        assert!(initial_revision < updated_revision);
        assert!(entity.header().revision() == updated_revision);
    }

    #[test]
    fn remove_entity() {
        let connection = establish_connection();
        let repository = CollectionRepository::new(&connection);
        let entity = repository.create_entity(CollectionBody { name: "Test Collection".into() }).unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        repository.remove_entity(&entity.header().uid()).unwrap();
        println!("Removed entity: {}", entity.header().uid());
    }
}
