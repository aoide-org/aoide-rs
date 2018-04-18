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

use domain::entity::*;
use domain::collection::*;

///////////////////////////////////////////////////////////////////////
/// CollectionRecord
///////////////////////////////////////////////////////////////////////

#[derive(Insertable)]
#[table_name = "collection_entity"]
pub struct InsertableCollectionEntity<'a> {
    pub uid: &'a str,
    pub revno: i64,
    pub revts: NaiveDateTime,
    pub name: &'a str,
}

impl<'a> InsertableCollectionEntity<'a> {
    pub fn from_entity(entity: &'a CollectionEntity) -> Self {
        Self {
            uid: entity.header().uid().as_str(),
            revno: entity.header().revision().number() as i64,
            revts: entity.header().revision().timestamp().naive_utc(),
            name: &entity.name(),
        }
    }
}

#[derive(AsChangeset)]
#[table_name = "collection_entity"]
pub struct UpdatableCollectionEntity<'a> {
    pub revno: i64,
    pub revts: NaiveDateTime,
    pub name: &'a str,
}

impl<'a> UpdatableCollectionEntity<'a> {
    pub fn from_entity_revision(entity: &'a CollectionEntity, revision: EntityRevision) -> Self {
        Self {
            revno: revision.number() as i64,
            revts: revision.timestamp().naive_utc(),
            name: &entity.name(),
        }
    }
}

#[derive(Queryable)]
pub struct QueryableCollectionEntity {
    pub id: i64,
    pub uid: String,
    pub revno: i64,
    pub revts: NaiveDateTime,
    pub name: String,
}

impl Into<CollectionEntity> for QueryableCollectionEntity {
    fn into(self) -> CollectionEntity {
        let uid: EntityUid = self.uid.into();
        let revision = EntityRevision::new(self.revno as u64, DateTime::from_utc(self.revts, Utc));
        let header = EntityHeader::new(uid, revision);
        CollectionEntity::new(header, self.name)
    }
}

///////////////////////////////////////////////////////////////////////
/// CollectionRepository
///////////////////////////////////////////////////////////////////////

pub struct CollectionRepository {
    connection: diesel::SqliteConnection,
}

impl CollectionRepository {
    pub fn new(connection: diesel::SqliteConnection) -> Self {
        Self { connection }
    }

    pub fn find_entity_by_uid(&self, uid: &EntityUid) -> Result<Option<CollectionEntity>, diesel::result::Error> {
        let target =
            collection_entity::table.filter(collection_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableCollectionEntity>(&self.connection)
            .optional()?;
        Ok(result.map(|r| r.into()))
    }

    pub fn find_entities_by_name(&self, name: &str) -> Result<Vec<CollectionEntity>, diesel::result::Error> {
        let target =
            collection_entity::table.filter(collection_entity::name.eq(name));
        let results = target
            .load::<QueryableCollectionEntity>(&self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    pub fn find_entities_by_name_starting_with(&self, name_prefix: &str) -> Result<Vec<CollectionEntity>, diesel::result::Error> {
        let target =
            collection_entity::table.filter(collection_entity::name.like(format!("{}%", name_prefix)));
        let results = target
            .load::<QueryableCollectionEntity>(&self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    pub fn find_entities_by_name_containing(&self, partial_name: &str) -> Result<Vec<CollectionEntity>, diesel::result::Error> {
        let target =
            collection_entity::table.filter(collection_entity::name.like(format!("%{}%", partial_name)));
        let results = target
            .load::<QueryableCollectionEntity>(&self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    pub fn create_entity<S: Into<String>>(&self, name: S) -> Result<CollectionEntity, diesel::result::Error> {
        let entity = CollectionEntity::with_name(name);
        {
            let insertable = InsertableCollectionEntity::from_entity(&entity);
            let query = diesel::insert_into(collection_entity::table).values(&insertable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            query.execute(&self.connection)?;
        }
        if log_enabled!(log::Level::Debug) {
            debug!("Created collection entity: {:?}", entity.header());
        }
        Ok(entity)
    }

    pub fn update_entity(&self, entity: &mut CollectionEntity) -> Result<EntityRevision, diesel::result::Error> {
        let next_revision = entity.header().revision().next();
        {
            let updatable = UpdatableCollectionEntity::from_entity_revision(&entity, next_revision);
            let target =
                collection_entity::table.filter(collection_entity::uid.eq(entity.header().uid().as_str()));
            let query = diesel::update(target).set(&updatable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            query.execute(&self.connection)?;
        }
        entity.update_revision(next_revision);
        if log_enabled!(log::Level::Debug) {
            debug!("Updated collection entity: {:?}", entity.header());
        }
        Ok(next_revision)
    }

    pub fn remove_entity(&self, uid: &EntityUid) -> Result<(), diesel::result::Error> {
        let target = collection_entity::table.filter(collection_entity::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Executing SQLite query: {}",
                diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
            );
        }
        query.execute(&self.connection)?;
        if log_enabled!(log::Level::Debug) {
            debug!("Removed collection entity: {}", uid);
        }
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    fn establish_connection() -> SqliteConnection {
        SqliteConnection::establish(":memory:")
            .expect("Failed to create in-memory connection for testing")
        // TODO: Init schema
    }

    fn new_repository() -> CollectionRepository {
        CollectionRepository::new(establish_connection())
    }

    #[test]
    #[ignore] // TODO: Enable when schema for test connection has been initialized
    fn create_entity() {
        let repository = new_repository();
        let entity = repository.create_entity("Test Collection").unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
    }

    #[test]
    #[ignore] // TODO: Enable when schema for test connection has been initialized
    fn update_entity() {
        let repository = new_repository();
        let mut entity = repository.create_entity("Test Collection").unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        let initial_revision = entity.header().revision();
        entity.set_name("Renamed Collection");
        let updated_revision = repository.update_entity(&mut entity).unwrap();
        println!("Updated entity: {:?}", entity);
        assert!(initial_revision < updated_revision);
        assert!(entity.header().revision() == updated_revision);
    }

    #[test]
    #[ignore] // TODO: Enable when schema for test connection has been initialized
    fn remove_entity() {
        let repository = new_repository();
        let entity = repository.create_entity("Test Collection").unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        repository.remove_entity(&entity.header().uid()).unwrap();
        println!("Removed entity: {}", entity.header().uid());
    }
}
