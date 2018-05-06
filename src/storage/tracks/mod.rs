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

use diesel::prelude::*;
use diesel;

use failure;

use log;

use std::i64;

use super::*;
use super::serde::{serialize_with_format, SerializationFormat, SerializedEntity};

use usecases::{Tracks, TracksResult};
use usecases::search::SearchParams;
use usecases::result::Pagination;

use aoide_core::domain::track::*;

///////////////////////////////////////////////////////////////////////
/// TrackRepository
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Queryable, Identifiable)]
#[table_name = "tracks_entity"]
pub struct QueryableSerializedEntity {
    pub id: StorageId,
    pub uid: String,
    pub rev_ordinal: i64,
    pub rev_timestamp: NaiveDateTime,
    pub ser_fmt: i16,
    pub ser_ver_major: i32,
    pub ser_ver_minor: i32,
    pub ser_blob: Vec<u8>,
}

impl From<QueryableSerializedEntity> for SerializedEntity {
    fn from(from: QueryableSerializedEntity) -> Self {
        let uid: EntityUid = from.uid.into();
        let revision = EntityRevision::new(
            from.rev_ordinal as u64,
            DateTime::from_utc(from.rev_timestamp, Utc),
        );
        let header = EntityHeader::new(uid, revision);
        let format = SerializationFormat::from(from.ser_fmt).unwrap();
        assert!(from.ser_ver_major >= 0);
        assert!(from.ser_ver_minor >= 0);
        let version = EntityVersion::new(from.ser_ver_major as u32, from.ser_ver_minor as u32);
        SerializedEntity {
            header,
            format,
            version,
            blob: from.ser_blob,
        }
    }
}

pub struct TrackRepository<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> TrackRepository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }

    pub fn cleanup_aux_storage(&self) -> Result<(), failure::Error> {
        self.cleanup_aux_identity()?;
        self.cleanup_aux_overview()?;
        self.cleanup_aux_summary()?;
        self.cleanup_aux_resource()?;
        self.cleanup_aux_music()?;
        self.cleanup_aux_tag()?;
        self.cleanup_aux_comment()?;
        self.cleanup_aux_rating()?;
        Ok(())
    }

    pub fn refresh_aux_storage(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        self.delete_aux_storage(track_id)?;
        self.insert_aux_storage(track_id, track_body)?;
        Ok(())
    }

    fn insert_aux_storage(
        &self,
        storage_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        self.insert_aux_identity(storage_id, track_body)?;
        self.insert_aux_overview(storage_id, track_body)?;
        self.insert_aux_summary(storage_id, track_body)?;
        self.insert_aux_resource(storage_id, track_body)?;
        self.insert_aux_music(storage_id, track_body)?;
        self.insert_aux_tag(storage_id, track_body)?;
        self.insert_aux_comment(storage_id, track_body)?;
        self.insert_aux_rating(storage_id, track_body)?;
        Ok(())
    }

    fn delete_aux_storage(&self, track_id: StorageId) -> Result<(), failure::Error> {
        self.delete_aux_identity(track_id)?;
        self.delete_aux_overview(track_id)?;
        self.delete_aux_summary(track_id)?;
        self.delete_aux_resource(track_id)?;
        self.delete_aux_music(track_id)?;
        self.delete_aux_tag(track_id)?;
        self.delete_aux_comment(track_id)?;
        self.delete_aux_rating(track_id)?;
        Ok(())
    }

    fn cleanup_aux_identity(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_identity::table.filter(
            aux_tracks_identity::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_identity(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_identity::table.filter(aux_tracks_identity::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_identity(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        let insertable = InsertableTracksIdentity::bind(track_id, track_body);
        let query = diesel::insert_into(aux_tracks_identity::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_aux_overview(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_overview::table.filter(
            aux_tracks_overview::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_overview(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_overview::table.filter(aux_tracks_overview::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_overview(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        let insertable = InsertableTracksOverview::bind(track_id, track_body);
        let query = diesel::insert_into(aux_tracks_overview::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_aux_summary(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_summary::table.filter(
            aux_tracks_summary::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_summary(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_summary::table.filter(aux_tracks_summary::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_summary(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        let insertable = InsertableTracksSummary::bind(track_id, track_body);
        let query = diesel::insert_into(aux_tracks_summary::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_aux_resource(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_resource::table.filter(
            aux_tracks_resource::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_resource(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_resource::table.filter(aux_tracks_resource::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_resource(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        for resource in track_body.resources.iter() {
            let insertable = InsertableTracksResource::bind(track_id, resource);
            let query = diesel::insert_into(aux_tracks_resource::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_aux_music(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_music::table.filter(
            aux_tracks_music::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_music(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_music::table.filter(aux_tracks_music::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_music(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        if track_body.music.is_some() {
            let insertable = InsertableTracksMusic::bind(track_id, track_body.music.as_ref().unwrap());
            let query = diesel::insert_into(aux_tracks_music::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_aux_tag(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_tag::table.filter(
            aux_tracks_tag::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_tag(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query =
            diesel::delete(aux_tracks_tag::table.filter(aux_tracks_tag::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_tag(&self, track_id: StorageId, track_body: &TrackBody) -> Result<(), failure::Error> {
        for tag in track_body.tags.iter() {
            let insertable = InsertableTracksTag::bind(track_id, tag);
            let query = diesel::insert_into(aux_tracks_tag::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_aux_comment(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_comment::table.filter(
            aux_tracks_comment::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_comment(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_comment::table.filter(aux_tracks_comment::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_comment(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        for comment in track_body.comments.iter() {
            let insertable = InsertableTracksComment::bind(track_id, comment);
            let query = diesel::insert_into(aux_tracks_comment::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_aux_rating(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_rating::table.filter(
            aux_tracks_rating::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_rating(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query = diesel::delete(
            aux_tracks_rating::table.filter(aux_tracks_rating::track_id.eq(track_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_rating(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        for rating in track_body.ratings.iter() {
            let insertable = InsertableTracksRating::bind(track_id, rating);
            let query = diesel::insert_into(aux_tracks_rating::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn after_entity_created(&self, entity: &TrackEntity) -> Result<StorageId, failure::Error> {
        let uid = entity.header().uid();
        let maybe_storage_id = self.find_storage_id(uid)?;
        match maybe_storage_id {
            Some(storage_id) => {
                self.insert_aux_storage(storage_id, entity.body())?;
                Ok(storage_id)
            }
            None => Err(format_err!("Entity not found: {}", uid)),
        }
    }

    fn before_entity_updated_or_removed(
        &self,
        uid: &EntityUid,
    ) -> Result<StorageId, failure::Error> {
        let maybe_storage_id = self.find_storage_id(uid)?;
        match maybe_storage_id {
            Some(storage_id) => {
                self.delete_aux_storage(storage_id)?;
                Ok(storage_id)
            }
            None => Err(format_err!("Entity not found: {}", uid)),
        }
    }

    fn after_entity_updated(
        &self,
        storage_id: StorageId,
        body: &TrackBody,
    ) -> Result<(), failure::Error> {
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
            let entity_blob = serialize_with_format(&entity, format)?;
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
            let entity_blob = serialize_with_format(&entity, format)?;
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
            Some(ref uid) => target
                .filter(
                    tracks_entity::id.eq_any(
                        aux_tracks_resource::table
                            .select(aux_tracks_resource::track_id)
                            .filter(aux_tracks_resource::collection_uid.eq(uid.as_str())),
                    ),
                )
                .load::<QueryableSerializedEntity>(self.connection),
            None => target.load::<QueryableSerializedEntity>(self.connection),
        }?;
        if log_enabled!(log::Level::Debug) {
            debug!("Loaded {} track entities", results.len(),);
        }
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        search_params: &SearchParams,
        pagination: &Pagination,
    ) -> Result<Vec<SerializedEntity>, failure::Error> {
        let offset = pagination.offset.map(|offset| offset as i64).unwrap_or(0);
        let limit = pagination
            .limit
            .map(|limit| limit as i64)
            .unwrap_or(i64::MAX);
        let like_expr = search_params.filter.as_ref().map(|s| format!("%{}%", s)).unwrap_or("%".into());
        let target = tracks_entity::table
            .left_outer_join(aux_tracks_resource::table)
            .left_outer_join(aux_tracks_overview::table)
            .left_outer_join(aux_tracks_summary::table)
            .left_outer_join(aux_tracks_music::table)
            .filter(aux_tracks_overview::track_title.like(&like_expr))
            .or_filter(aux_tracks_overview::album_title.like(&like_expr))
            .or_filter(aux_tracks_summary::track_artists.like(&like_expr))
            .or_filter(aux_tracks_summary::album_artists.like(&like_expr))
            .or_filter(
                tracks_entity::id.eq_any(
                    aux_tracks_tag::table
                        .select(aux_tracks_tag::track_id)
                        .filter(aux_tracks_tag::facet.eq(TrackTag::FACET_GENRE))
                        .filter(aux_tracks_tag::term.like(&like_expr)),
                ),
            )
            .or_filter(
                tracks_entity::id.eq_any(
                    aux_tracks_comment::table
                        .select(aux_tracks_comment::track_id)
                        .filter(aux_tracks_comment::comment.like(&like_expr)),
                ),
            )
            .select(tracks_entity::all_columns)
            .offset(offset)
            .limit(limit);
        let results = match collection_uid {
            Some(ref uid) => target
                .filter(aux_tracks_resource::collection_uid.eq(uid.as_str()))
                .order(aux_tracks_resource::collection_since.desc()) // recently added to collection
                .load::<QueryableSerializedEntity>(self.connection),
            None => target
                .order(tracks_entity::rev_timestamp.desc()) // recently modified
                .load::<QueryableSerializedEntity>(self.connection),
        }?;
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
