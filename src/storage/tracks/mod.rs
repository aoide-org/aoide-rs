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

use log;

use rmp_serde;

use storage::*;

use usecases::*;

use aoide_core::domain::track::*;

///////////////////////////////////////////////////////////////////////
/// TrackRepository
///////////////////////////////////////////////////////////////////////

pub struct TrackRepository<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> TrackRepository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }
}

impl<'a> EntityStorage for TrackRepository<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        let target = tracks_entity::table
            .select(TRACKS_ENTITY_ID_COLUMN)
            .filter(tracks_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableStorageId>(self.connection)
            .optional()?;
        Ok(result.map(|r| r.id))
    }
}

impl<'a> Tracks for TrackRepository<'a> {
    fn create_entity(&self, body: TrackBody) -> TracksResult<TrackEntity> {
        let entity = TrackEntity::with_body(body);
        {
            let entity_blob = rmp_serde::to_vec(&entity)?;
            let insertable = InsertableTracksEntity::bind(entity.header(), SerializationFormat::MessagePack, &entity_blob);
            let query = diesel::insert_into(tracks_entity::table).values(&insertable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            query.execute(self.connection)?;
        }
        if log_enabled!(log::Level::Debug) {
            debug!("Created track entity: {:?}", entity.header());
        }
        Ok(entity)
    }

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableTracksEntity>(self.connection)
            .optional()?;
        if log_enabled!(log::Level::Debug) {
            match &result {
                &None => {
                    debug!("Found no track entity with uid '{}'", uid);
                }
                &Some(_) => {
                    debug!("Found a track entity with uid '{}'", uid);
                }
            }
        }
        Ok(result.map(|r| r.into()))

    }

    /*
    fn update_entity(&self, entity: &TrackEntity) -> TracksResult<Option<EntityRevision>> {
        let next_revision = entity.header().revision().next();
        {
            let updatable = UpdatableTrackEntity::bind(&next_revision, &entity.body());
            let target = tracks_entity::table
                .filter(tracks_entity::uid.eq(entity.header().uid().as_str())
                    .and(tracks_entity::rev_ordinal.eq(entity.header().revision().ordinal() as i64))
                    .and(tracks_entity::rev_timestamp.eq(entity.header().revision().timestamp().naive_utc())));
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
            debug!("Updated track entity: {:?} -> {:?}", entity.header(), next_revision);
        }
        Ok(Some(next_revision))
    }

    fn remove_entity(&self, uid: &EntityUid) -> TracksResult<Option<()>> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
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
            debug!("Removed track entity: {}", uid);
        }
        Ok(Some(()))
    }

    fn find_entity(&self, uid: &EntityUid) -> TracksResult<Option<TrackEntity>> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableTrackEntity>(self.connection)
            .optional()?;
        if log_enabled!(log::Level::Debug) {
            match &result {
                &None => {
                    debug!("Found no track entity with uid '{}'", uid);
                }
                &Some(_) => {
                    debug!("Found a track entity with uid '{}'", uid);
                }
            }
        }
        Ok(result.map(|r| r.into()))
    }
    */
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

    /*
    #[test]
    fn create_entity() {
        let connection = establish_connection();
        let repository = TrackRepository::new(&connection);
        let entity = repository
            .create_entity(TrackBody {
                name: "Test Track".into(),
                description: Some("Description".into()),
            })
            .unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
    }

    #[test]
    fn update_entity() {
        let connection = establish_connection();
        let repository = TrackRepository::new(&connection);
        let mut entity = repository
            .create_entity(TrackBody {
                name: "Test Track".into(),
                description: Some("Description".into()),
            })
            .unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        let prev_revision = entity.header().revision();
        entity.body_mut().name = "Renamed Track".into();
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
        let repository = TrackRepository::new(&connection);
        let entity = repository
            .create_entity(TrackBody {
                name: "Test Track".into(),
                description: None,
            })
            .unwrap();
        println!("Created entity: {:?}", entity);
        assert!(entity.is_valid());
        assert!(Some(()) == repository.remove_entity(&entity.header().uid()).unwrap());
        println!("Removed entity: {}", entity.header().uid());
    }
    */
}
