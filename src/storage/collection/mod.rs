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
use self::schema::collection;

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
#[table_name = "collection"]
pub struct InsertableCollectionRecord<'a> {
    pub uid: &'a str,
    pub revno: i64,
    pub revts: NaiveDateTime,
    pub name: &'a str,
}

impl<'a> InsertableCollectionRecord<'a> {
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
#[table_name = "collection"]
pub struct UpdatableCollectionRecord<'a> {
    pub revno: i64,
    pub revts: NaiveDateTime,
    pub name: &'a str,
}

impl<'a> UpdatableCollectionRecord<'a> {
    pub fn from_entity_revision(entity: &'a CollectionEntity, revision: EntityRevision) -> Self {
        Self {
            revno: revision.number() as i64,
            revts: revision.timestamp().naive_utc(),
            name: &entity.name(),
        }
    }
}

#[derive(Queryable)]
pub struct QueryableCollectionRecord {
    pub id: u64,
    pub uid: String,
    pub revno: u64,
    pub revts: NaiveDateTime,
    pub name: String,
}

impl QueryableCollectionRecord {
    pub fn into_entity(&self) -> CollectionEntity {
        let revision = EntityRevision::new(self.revno, DateTime::from_utc(self.revts, Utc));
        let header = EntityHeader::new(self.uid.clone(), revision);
        CollectionEntity::new(header, self.name.clone())
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

    pub fn create_entity<S: Into<String>>(&self, name: S) -> CollectionEntity {
        let entity = CollectionEntity::with_name(name);
        {
            let record = InsertableCollectionRecord::from_entity(&entity);
            let query = diesel::insert_into(collection::table).values(&record);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            /*
            query
                .execute(&self.connection)
                .expect("Error inserting record for newly created entity");
            */
        }
        if log_enabled!(log::Level::Debug) {
            debug!("Created collection entity: {:?}", entity.header());
        }
        entity
    }

    pub fn update_entity(&self, entity: &mut CollectionEntity) -> EntityRevision {
        let next_revision = entity.header().revision().next();
        {
            let record = UpdatableCollectionRecord::from_entity_revision(&entity, next_revision);
            let target = collection::table.filter(collection::uid.eq(entity.header().uid().as_str()));
            let query = diesel::update(target).set(&record);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            /*
            query
                .execute(&self.connection)
                .expect("Error updating record of modified entity");
            */
        }
        entity.update_revision(next_revision);
        if log_enabled!(log::Level::Debug) {
            debug!("Updated collection entity: {:?}", entity.header());
        }
        next_revision
    }

    pub fn remove_entity(&self, uid: &EntityUid) -> bool {
        let target = collection::table.filter(collection::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Executing SQLite query: {}",
                diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
            );
        }
        /*
        query
            .execute(&self.connection)
            .expect("Error deleting record of entity");
        */
        if log_enabled!(log::Level::Debug) {
            debug!("Removed collection entity: {}", uid);
        }
        false
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
    fn create_entity() {
        let repository = new_repository();
        let entity = repository.create_entity("Test Collection");
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
    }

    #[test]
    fn update_entity() {
        let repository = new_repository();
        let mut entity = repository.create_entity("Test Collection");
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        let initial_revision = entity.header().revision();
        entity.set_name("Renamed Collection");
        let updated_revision = repository.update_entity(&mut entity);
        println!("Updated entity: {:?}", entity);
        assert!(initial_revision < updated_revision);
        assert!(entity.header().revision() == updated_revision);
    }

    #[test]
    fn remove_entity() {
        let repository = new_repository();
        let entity = repository.create_entity("Test Collection");
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        repository.remove_entity(&entity.header().uid());
        println!("Removed entity: {}", entity.header().uid());
    }
}
