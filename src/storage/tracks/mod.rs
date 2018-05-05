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

use storage::*;

use usecases::*;

use aoide_core::domain::track::*;
use aoide_core::domain::metadata::{Tag, Comment, Rating};

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

    pub fn cleanup_aux_storage(&self) -> Result<(), failure::Error> {
        self.cleanup_aux_resources()?;
        self.cleanup_aux_tags()?;
        self.cleanup_aux_comments()?;
        self.cleanup_aux_ratings()?;
        Ok(())
    }

    pub fn refresh_aux_storage(&self, track_id: StorageId, track_body: &TrackBody) -> Result<(), failure::Error> {
        self.delete_aux_storage(track_id)?;
        self.insert_aux_storage(track_id, track_body)?;
        Ok(())
    }

    fn insert_aux_storage(&self, storage_id: StorageId, track_body: &TrackBody) -> Result<(), failure::Error> {
        for resource in track_body.resources.iter() {
            self.insert_aux_resource(storage_id, resource)?;
        }
        for tag in track_body.tags.iter() {
            self.insert_aux_tag(storage_id, tag)?;
        }
        for comment in track_body.comments.iter() {
            self.insert_aux_comment(storage_id, comment)?;
        }
        for rating in track_body.ratings.iter() {
            self.insert_aux_rating(storage_id, rating)?;
        }
        Ok(())
    }

    fn delete_aux_storage(&self, track_id: StorageId) -> Result<(), failure::Error> {
        self.delete_aux_resources(track_id)?;
        self.delete_aux_tags(track_id)?;
        self.delete_aux_comments(track_id)?;
        self.delete_aux_ratings(track_id)?;
        Ok(())
    }

    fn cleanup_aux_resources(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_resource::table.filter(aux_tracks_resource::track_id.ne_all(
            tracks_entity::table.select(tracks_entity::id))));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_resources(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_resource::table.filter(aux_tracks_resource::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_resource(&self, track_id: StorageId, collection: &TrackResource) -> Result<(), failure::Error> {
        let insertable = InsertableTracksResource::bind(track_id, collection);
        let query = diesel::insert_into(aux_tracks_resource::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_aux_tags(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_tag::table.filter(aux_tracks_tag::track_id.ne_all(
            tracks_entity::table.select(tracks_entity::id))));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_tags(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_tag::table.filter(aux_tracks_tag::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_tag(&self, track_id: StorageId, tag: &Tag) -> Result<(), failure::Error> {
        let insertable = InsertableTracksTag::bind(track_id, tag);
        let query = diesel::insert_into(aux_tracks_tag::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_aux_comments(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_comment::table.filter(aux_tracks_comment::track_id.ne_all(
            tracks_entity::table.select(tracks_entity::id))));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_comments(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_comment::table.filter(aux_tracks_comment::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_comment(&self, track_id: StorageId, comment: &Comment) -> Result<(), failure::Error> {
        let insertable = InsertableTracksComment::bind(track_id, comment);
        let query = diesel::insert_into(aux_tracks_comment::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_aux_ratings(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_rating::table.filter(aux_tracks_rating::track_id.ne_all(
            tracks_entity::table.select(tracks_entity::id))));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_ratings(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_rating::table.filter(aux_tracks_rating::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_rating(&self, track_id: StorageId, rating: &Rating) -> Result<(), failure::Error> {
        let insertable = InsertableTracksRating::bind(track_id, rating);
        let query = diesel::insert_into(aux_tracks_rating::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn after_entity_created(&self, entity: &TrackEntity) -> Result<StorageId, failure::Error> {
        let uid = entity.header().uid();
        let maybe_storage_id = self.find_storage_id(uid)?;
        match maybe_storage_id {
            Some(storage_id) => {
                self.insert_aux_storage(storage_id, entity.body())?;
                Ok(storage_id)
            },
            None => Err(format_err!("Entity not found: {}", uid))
        }
    }

    fn before_entity_updated_or_removed(&self, uid: &EntityUid) -> Result<StorageId, failure::Error> {
        let maybe_storage_id = self.find_storage_id(uid)?;
        match maybe_storage_id {
            Some(storage_id) => {
                self.delete_aux_storage(storage_id)?;
                Ok(storage_id)
            },
            None => Err(format_err!("Entity not found: {}", uid))
        }
    }

    fn after_entity_updated(&self, storage_id: StorageId, body: &TrackBody) -> Result<(), failure::Error> {
        self.insert_aux_storage(storage_id, body)?;
        Ok(())
    }
}

impl<'a> EntityStorage for TrackRepository<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        let target = tracks_entity::table
            .select((tracks_entity::id,))
            .filter(tracks_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableStorageId>(self.connection)
            .optional()?;
        Ok(result.map(|r| r.id))
    }
}

impl<'a> Tracks for TrackRepository<'a> {
    fn create_entity(
        &self,
        body: TrackBody,
        format: SerializationFormat,
    ) -> TracksResult<TrackEntity> {
        let entity = TrackEntity::with_body(body);
        {
            let entity_blob = serialize_entity(&entity, format)?;
            let insertable = InsertableTracksEntity::bind(entity.header(), format, &entity_blob);
            let query = diesel::insert_into(tracks_entity::table).values(&insertable);
            if log_enabled!(log::Level::Debug) {
                debug!(
                    "Executing SQLite query: {}",
                    diesel::debug_query::<diesel::sqlite::Sqlite, _>(&query)
                );
            }
            query.execute(self.connection)?;
        }
        self.after_entity_created(&entity)?;
        if log_enabled!(log::Level::Debug) {
            debug!("Created track entity: {:?}", entity.header());
        }
        Ok(entity)
    }

    fn update_entity(
        &self,
        entity: &mut TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<Option<()>> {
        let prev_revision = entity.header().revision();
        let next_revision = prev_revision.next();
        {
            entity.update_revision(next_revision);
            let entity_blob = serialize_entity(&entity, format)?;
            {
                let updatable = UpdatableTracksEntity::bind(&next_revision, format, &entity_blob);
                let uid = entity.header().uid();
                let target = tracks_entity::table.filter(
                    tracks_entity::uid
                        .eq(uid.as_str())
                        .and(tracks_entity::rev_ordinal.eq(prev_revision.ordinal() as i64))
                        .and(
                            tracks_entity::rev_timestamp.eq(prev_revision.timestamp().naive_utc()),
                        ),
                );
                let storage_id = self.before_entity_updated_or_removed(uid)?;
                let query = diesel::update(target).set(&updatable);
                let rows_affected: usize = query.execute(self.connection)?;
                assert!(rows_affected <= 1);
                if rows_affected <= 0 {
                    return Ok(None);
                }
                self.after_entity_updated(storage_id, &entity.body())?;
            }
        }
        if log_enabled!(log::Level::Debug) {
            debug!(
                "Updated track entity: {:?} -> {:?}",
                entity.header(),
                next_revision
            );
        }
        Ok(Some(()))
    }

    fn remove_entity(&self, uid: &EntityUid) -> TracksResult<()> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        self.before_entity_updated_or_removed(uid)?;
        let rows_affected: usize = query.execute(self.connection)?;
        assert!(rows_affected <= 1);
        Ok(())
    }

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableSerializedEntity>(self.connection)
            .optional()?;
        if log_enabled!(log::Level::Debug) {
            match &result {
                &None => {
                    debug!("Found no track entity with uid '{}'", uid);
                }
                &Some(_) => {
                    debug!("Loaded track entity with uid '{}'", uid);
                }
            }
        }
        Ok(result.map(|r| r.into()))
    }

    fn load_recently_revisioned_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
    ) -> TracksResult<Vec<SerializedEntity>> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination
            .limit
            .map(|limit| limit as i64)
            .unwrap_or(i64::MAX);
        let target = tracks_entity::table
            .order(tracks_entity::rev_timestamp.desc())
            .offset(offset)
            .limit(limit);
        let results = match collection_uid {
            Some(ref uid) => target.filter(tracks_entity::id.eq_any(aux_tracks_resource::table.select(aux_tracks_resource::track_id).filter(aux_tracks_resource::collection_uid.eq(uid.as_str())))).load::<QueryableSerializedEntity>(self.connection),
            None => target.load::<QueryableSerializedEntity>(self.connection),
        }?;
        if log_enabled!(log::Level::Debug) {
            debug!("Loaded {} track entities", results.len(),);
        }
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
