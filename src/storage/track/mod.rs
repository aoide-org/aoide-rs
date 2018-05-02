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

use self::schema::track_entity;
use self::schema::track_collection_resource;

use chrono::{DateTime, Utc};
use chrono::naive::NaiveDateTime;

use diesel::prelude::*;
use diesel;

use log;

use serde_json;

use aoide_core::domain::entity::*;
use aoide_core::domain::track::*;

use storage::*;

use usecases::*;

///////////////////////////////////////////////////////////////////////
/// TrackRecord
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Insertable)]
#[table_name = "track_entity"]
pub struct InsertableTrackEntity<'a> {
    pub uid: &'a str,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub entity_fmt: i16,
    pub entity_ver_major: i32,
    pub entity_ver_minor: i32,
    pub entity_blob: &'a [u8],
}

impl<'a> InsertableTrackEntity<'a> {
    pub fn borrow(entity: &'a TrackEntity, entity_blob: &'a [u8]) -> Self {
        Self {
            uid: entity.header().uid().as_str(),
            rev_ordinal: entity.header().revision().ordinal() as i64,
            rev_timestamp: entity.header().revision().timestamp().naive_utc(),
            entity_fmt: 1, // JSON
            entity_ver_major: 0, // TODO
            entity_ver_minor: 0, // TODO
            entity_blob,
        }
    }
}

#[derive(Debug, Insertable)]
#[table_name = "track_collection_resource"]
pub struct InsertableTrackCollectionResource<'a> {
    pub track_id: i64,
    pub collection_uid: &'a str,
    pub media_uri: &'a str,
    pub media_content_type: &'a str,
    pub media_sync_rev_ordinal: Option<i64>,
    pub media_sync_rev_timestamp: Option<NaiveDateTime>,
    pub audio_duration: Option<i64>,
    pub audio_channels: Option<i16>,
    pub audio_samplerate: Option<i32>,
    pub audio_bitrate: Option<i32>,
    pub audio_enc_name: Option<&'a str>,
    pub audio_enc_settings: Option<&'a str>,
}

impl<'a> InsertableTrackCollectionResource<'a> {
    pub fn borrow(track_id: StorageId, collected_resource: &'a CollectedMediaResource) -> Self {
        Self {
            track_id: track_id as i64,
            collection_uid: collected_resource.collection_uid.as_str(),
            media_uri: collected_resource.media_resource.uri.as_str(),
            media_content_type: collected_resource.media_resource.content_type.as_str(),
            media_sync_rev_ordinal: collected_resource.media_resource.synchronized_revision.map(|rev| rev.ordinal() as i64),
            media_sync_rev_timestamp: collected_resource.media_resource.synchronized_revision.map(|rev| rev.timestamp().naive_utc()),
            audio_duration: collected_resource.media_resource.audio_content.as_ref().map(|audio| audio.duration.millis as i64),
            audio_channels: collected_resource.media_resource.audio_content.as_ref().map(|audio| audio.channels.count as i16),
            audio_samplerate: collected_resource.media_resource.audio_content.as_ref().map(|audio| audio.samplerate.hz as i32),
            audio_bitrate: collected_resource.media_resource.audio_content.as_ref().map(|audio| audio.bitrate.bps as i32),
            audio_enc_name: collected_resource.media_resource.audio_content.as_ref().and_then(|audio| audio.encoder.as_ref()).map(|enc| enc.name.as_str()),
            audio_enc_settings: collected_resource.media_resource.audio_content.as_ref().and_then(|audio| audio.encoder.as_ref()).and_then(|enc| enc.settings.as_ref()).map(|settings| settings.as_str()),
        }
    }
}

/*
#[derive(Debug, AsChangeset)]
#[table_name = "track_entity"]
pub struct UpdatableTrackEntity<'a> {
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: &'a str,
    pub description: Option<&'a str>,
}

impl<'a> UpdatableTrackEntity<'a> {
    pub fn borrow(next_revision: &EntityRevision, body: &'a TrackBody) -> Self {
        Self {
            rev_ordinal: next_revision.ordinal() as i64,
            rev_timestamp: next_revision.timestamp().naive_utc(),
            name: &body.name,
            description: body.description.as_ref().map(|s| s.as_str()),
        }
    }
}

#[derive(Debug, Queryable)]
pub struct QueryableTrackEntity {
    pub id: i64,
    pub uid: String,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub name: String,
    pub description: Option<String>,
}

impl From<QueryableTrackEntity> for TrackEntity {
    fn from(from: QueryableTrackEntity) -> Self {
        let uid: EntityUid = from.uid.into();
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let body = TrackBody {
            name: from.name,
            description: from.description,
        };
        Self::new(header, body)
    }
}
*/

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

type IdColumn = (
    track_entity::id,
);

const ID_COLUMN: IdColumn = (
    track_entity::id,
);

impl<'a> EntityStorage for TrackRepository<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        let target = track_entity::table
            .select(ID_COLUMN)
            .filter(track_entity::uid.eq(uid.as_str()));
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
            let entity_blob = serde_json::to_vec(&entity)?;
            let insertable = InsertableTrackEntity::borrow(&entity, &entity_blob);
            let query = diesel::insert_into(track_entity::table).values(&insertable);
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

    /*
    fn update_entity(&self, entity: &TrackEntity) -> TracksResult<Option<EntityRevision>> {
        let next_revision = entity.header().revision().next();
        {
            let updatable = UpdatableTrackEntity::borrow(&next_revision, &entity.body());
            let target = track_entity::table
                .filter(track_entity::uid.eq(entity.header().uid().as_str())
                    .and(track_entity::rev_ordinal.eq(entity.header().revision().ordinal() as i64))
                    .and(track_entity::rev_timestamp.eq(entity.header().revision().timestamp().naive_utc())));
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
        let target = track_entity::table.filter(track_entity::uid.eq(uid.as_str()));
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
        let target = track_entity::table.filter(track_entity::uid.eq(uid.as_str()));
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
