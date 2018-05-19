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

use diesel;
use diesel::dsl::*;
use diesel::prelude::*;

use failure;

use std::i64;

use super::serde::{deserialize_with_format, serialize_with_format, SerializationFormat,
                   SerializedEntity};
use super::*;

use usecases::request::{LocateMatcher, LocateParams, ReplaceMode, ReplaceParams, SearchParams};
use usecases::result::Pagination;
use usecases::{TrackEntityReplacement, Tracks, TracksResult, TrackTags, TrackTagsResult};

use aoide_core::domain::metadata::*;
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
        debug_assert!(from.ser_ver_major >= 0);
        debug_assert!(from.ser_ver_minor >= 0);
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
        self.cleanup_aux_overview()?;
        self.cleanup_aux_summary()?;
        self.cleanup_aux_resource()?;
        self.cleanup_aux_music()?;
        self.cleanup_aux_ref()?;
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
        self.insert_aux_overview(storage_id, track_body)?;
        self.insert_aux_summary(storage_id, track_body)?;
        self.insert_aux_resource(storage_id, track_body)?;
        self.insert_aux_music(storage_id, track_body)?;
        self.insert_aux_ref(storage_id, track_body)?;
        self.insert_aux_tag(storage_id, track_body)?;
        self.insert_aux_comment(storage_id, track_body)?;
        self.insert_aux_rating(storage_id, track_body)?;
        Ok(())
    }

    fn delete_aux_storage(&self, track_id: StorageId) -> Result<(), failure::Error> {
        self.delete_aux_overview(track_id)?;
        self.delete_aux_summary(track_id)?;
        self.delete_aux_resource(track_id)?;
        self.delete_aux_music(track_id)?;
        self.delete_aux_ref(track_id)?;
        self.delete_aux_tag(track_id)?;
        self.delete_aux_comment(track_id)?;
        self.delete_aux_rating(track_id)?;
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
        let query =
            diesel::delete(aux_tracks_music::table.filter(aux_tracks_music::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_music(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        if track_body.music.is_some() {
            let insertable =
                InsertableTracksMusic::bind(track_id, track_body.music.as_ref().unwrap());
            let query = diesel::insert_into(aux_tracks_music::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    fn cleanup_aux_ref(&self) -> Result<(), failure::Error> {
        let query = diesel::delete(aux_tracks_ref::table.filter(
            aux_tracks_ref::track_id.ne_all(tracks_entity::table.select(tracks_entity::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_aux_ref(&self, track_id: StorageId) -> Result<(), failure::Error> {
        let query =
            diesel::delete(aux_tracks_ref::table.filter(aux_tracks_ref::track_id.eq(track_id)));
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_aux_ref(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
        for track_ref in track_body.refs.iter() {
            let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Track, &track_ref);
            let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
            query.execute(self.connection)?;
        }
        for actor in track_body.actors.iter() {
            for actor_ref in actor.refs.iter() {
                let insertable = InsertableTracksRef::bind(track_id, RefOrigin::TrackActor, &actor_ref);
                let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                query.execute(self.connection)?;
            }
        }
        if let Some(album) = track_body.album.as_ref() {
            for album_ref in album.refs.iter() {
                let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Album, &album_ref);
                let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                query.execute(self.connection)?;
            }
            for actor in album.actors.iter() {
                for actor_ref in actor.refs.iter() {
                    let insertable = InsertableTracksRef::bind(track_id, RefOrigin::AlbumActor, &actor_ref);
                    let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                    query.execute(self.connection)?;
                }
            }
            if let Some(release) = album.release.as_ref() { 
                for release_ref in release.refs.iter() {
                    let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Release, &release_ref);
                    let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                    query.execute(self.connection)?;
                }
            }
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

    fn insert_aux_tag(
        &self,
        track_id: StorageId,
        track_body: &TrackBody,
    ) -> Result<(), failure::Error> {
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
        match self.find_storage_id(uid)? {
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
        match self.find_storage_id(uid)? {
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
            query.execute(self.connection)?;
        }
        self.after_entity_created(&entity)?;
        Ok(entity)
    }

    fn update_entity(
        &self,
        entity: &mut TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<Option<(EntityRevision, EntityRevision)>> {
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
                debug_assert!(rows_affected <= 1);
                if rows_affected <= 0 {
                    return Ok(None);
                }
                self.after_entity_updated(storage_id, &entity.body())?;
            }
        }
        Ok(Some((prev_revision, next_revision)))
    }

    fn replace_entity(
        &self,
        collection_uid: Option<&EntityUid>,
        replace_params: ReplaceParams,
        format: SerializationFormat,
    ) -> TracksResult<TrackEntityReplacement> {
        let locate_params = LocateParams {
            uri: replace_params.uri.clone(),
            matcher: LocateMatcher::Exact,
        };
        let located_entities =
            self.locate_entities(collection_uid, &Pagination::default(), locate_params)?;
        if located_entities.len() > 1 {
            Ok(TrackEntityReplacement::FoundTooMany)
        } else {
            match located_entities.first() {
                Some(serialized_entity) => {
                    let mut entity = deserialize_with_format::<TrackEntity>(serialized_entity)?;
                    entity.replace_body(replace_params.body);
                    self.update_entity(&mut entity, format)?;
                    Ok(TrackEntityReplacement::Updated(entity))
                }
                None => {
                    match replace_params.mode {
                        ReplaceMode::UpdateOrCreate => {
                            if let Some(collection_uid) = collection_uid {
                                // Check consistency to avoid unique constraint violations
                                // when inserting into the database.
                                match replace_params.body.resource(collection_uid) {
                                    Some(resource) => {
                                        if resource.source.uri != replace_params.uri {
                                            let msg = format!("Mismatching track URI: expected = '{}', actual = '{}'", replace_params.uri, resource.source.uri);
                                            return Ok(TrackEntityReplacement::NotFound(Some(msg)));
                                        }
                                    }
                                    None => {
                                        let msg = format!(
                                            "Track does not belong to collection with URI '{}'",
                                            collection_uid
                                        );
                                        return Ok(TrackEntityReplacement::NotFound(Some(msg)));
                                    }
                                }
                            };
                            let entity = self.create_entity(replace_params.body, format)?;
                            Ok(TrackEntityReplacement::Created(entity))
                        }
                        ReplaceMode::UpdateOnly => Ok(TrackEntityReplacement::NotFound(None)),
                    }
                }
            }
        }
    }

    fn remove_entity(&self, uid: &EntityUid) -> TracksResult<()> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
        let query = diesel::delete(target);
        self.before_entity_updated_or_removed(uid)?;
        let rows_affected: usize = query.execute(self.connection)?;
        debug_assert!(rows_affected <= 1);
        Ok(())
    }

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>> {
        let target = tracks_entity::table.filter(tracks_entity::uid.eq(uid.as_str()));
        let result = target
            .first::<QueryableSerializedEntity>(self.connection)
            .optional()?;
        Ok(result.map(|r| r.into()))
    }

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
        locate_params: LocateParams,
    ) -> TracksResult<Vec<SerializedEntity>> {
        let mut target = tracks_entity::table
            .left_outer_join(aux_tracks_resource::table)
            .select(tracks_entity::all_columns)
            .into_boxed();

        // Locate filter
        let locate_uri = match locate_params.matcher {
            // Escape wildcard character with backslash (see below)
            LocateMatcher::Front => format!(
                "{}%",
                locate_params.uri.replace('\\', "\\\\").replace('%', "\\%")
            ),
            LocateMatcher::Back => format!(
                "%{}",
                locate_params.uri.replace('\\', "\\\\").replace('%', "\\%")
            ),
            LocateMatcher::Partial => format!(
                "%{}%",
                locate_params.uri.replace('\\', "\\\\").replace('%', "\\%")
            ),
            LocateMatcher::Exact => locate_params.uri,
        };
        target = match locate_params.matcher {
            LocateMatcher::Exact => target.filter(aux_tracks_resource::source_uri.eq(locate_uri)),
            _ => target.filter(
                aux_tracks_resource::source_uri
                    .like(locate_uri)
                    .escape('\\'),
            ),
        };

        // Collection filter & ordering
        target = match collection_uid {
            Some(collection_uid) => target
                .filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))
                .order(aux_tracks_resource::collection_since.desc()), // recently added to collection
            None => target.order(tracks_entity::rev_timestamp.desc()), // recently modified
        };

        // Pagination
        if let Some(offset) = pagination.offset {
            target = target.offset(offset as i64);
        };
        if let Some(limit) = pagination.limit {
            target = target.limit(limit as i64);
        };

        let results: Vec<QueryableSerializedEntity> = target.load(self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
        search_params: SearchParams,
    ) -> TracksResult<Vec<SerializedEntity>> {

        // TODO: if/else arms are incompatible due to joining tables?
        let results = if search_params.filter.is_empty() {
            // Select all (without joining)
            let mut target = tracks_entity::table
                .select(tracks_entity::all_columns)
                .left_outer_join(aux_tracks_resource::table)
                .into_boxed();

            // Collection filter & ordering
            target = match collection_uid {
                Some(uid) => target
                    .filter(aux_tracks_resource::collection_uid.eq(uid.as_str()))
                    .order(aux_tracks_resource::collection_since.desc()), // recently added to collection
                None => target.order(tracks_entity::rev_timestamp.desc()), // recently modified
            };

            // Pagination
            if let Some(offset) = pagination.offset {
                target = target.offset(offset as i64);
            };
            if let Some(limit) = pagination.limit {
                target = target.limit(limit as i64);
            };

            target.load::<QueryableSerializedEntity>(self.connection)?
        } else {
            // Escape wildcard character with backslash (see below)
            let escaped_filter = search_params
                .filter
                .trim()
                .replace('\\', "\\\\")
                .replace('%', "\\%");
            let split_filter = escaped_filter.split_whitespace();
            let like_expr_len = split_filter
                .clone()
                .fold(1, |len, part| len + part.len() + 1);
            let mut like_expr = split_filter.fold(
                String::with_capacity(like_expr_len),
                |mut like_expr, part| {
                    // Prepend wildcard character before each part
                    like_expr.push('%');
                    like_expr.push_str(part);
                    like_expr
                },
            );
            // Append final wildcard character after last part
            like_expr.push('%');

            let mut target = tracks_entity::table
                .select(tracks_entity::all_columns)
                .left_outer_join(aux_tracks_resource::table)
                .left_outer_join(aux_tracks_overview::table)
                .left_outer_join(aux_tracks_summary::table)
                .left_outer_join(aux_tracks_music::table)
                .filter(
                    aux_tracks_overview::track_title
                        .like(&like_expr)
                        .escape('\\'),
                )
                .or_filter(
                    aux_tracks_overview::album_title
                        .like(&like_expr)
                        .escape('\\'),
                )
                .or_filter(
                    aux_tracks_summary::track_artist
                        .like(&like_expr)
                        .escape('\\'),
                )
                .or_filter(
                    aux_tracks_summary::album_artist
                        .like(&like_expr)
                        .escape('\\'),
                )
                .or_filter(
                    tracks_entity::id.eq_any(
                        aux_tracks_tag::table
                            .select(aux_tracks_tag::track_id)
                            .filter(aux_tracks_tag::facet.eq(TrackTag::FACET_GENRE))
                            .filter(aux_tracks_tag::term.like(&like_expr).escape('\\')),
                    ),
                )
                .or_filter(
                    tracks_entity::id.eq_any(
                        aux_tracks_comment::table
                            .select(aux_tracks_comment::track_id)
                            .filter(aux_tracks_comment::comment.like(&like_expr).escape('\\')),
                    ),
                )
                .into_boxed();

            // Collection filter & ordering
            target = match collection_uid {
                Some(uid) => target
                    .filter(aux_tracks_resource::collection_uid.eq(uid.as_str()))
                    .order(aux_tracks_resource::collection_since.desc()), // recently added to collection
                None => target.order(tracks_entity::rev_timestamp.desc()), // recently modified
            };

            // Pagination
            if let Some(offset) = pagination.offset {
                target = target.offset(offset as i64);
            };
            if let Some(limit) = pagination.limit {
                target = target.limit(limit as i64);
            };

            target.load::<QueryableSerializedEntity>(self.connection)?
        };
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

impl<'a> TrackTags for TrackRepository<'a> {

    fn all_tag_facets(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
    ) -> TrackTagsResult<Vec<TagFacetCount>> {
        let mut target = aux_tracks_tag::table
            .select((aux_tracks_tag::facet, sql::<diesel::sql_types::BigInt>("count(*) AS count")))
            .group_by(aux_tracks_tag::facet)
            .order(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Pagination
        if let Some(offset) = pagination.offset {
            target = target.offset(offset as i64);
        };
        if let Some(limit) = pagination.limit {
            target = target.limit(limit as i64);
        };

        if let Some(collection_uid) = collection_uid {
            let target = target
                .inner_join(aux_tracks_resource::table.on(
                    aux_tracks_tag::track_id.eq(aux_tracks_resource::track_id)
                    .and(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))
            ));
            let rows = target.load::<(Option<String>, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagFacetCount {
                    facet: row.0, count: row.1 as usize
                });
            }

            Ok(result)
        } else {
            let rows = target.load::<(Option<String>, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagFacetCount {
                    facet: row.0, count: row.1 as usize
                });
            }

            Ok(result)
        }

    }

    fn all_tag_terms(
        &self,
        collection_uid: Option<&EntityUid>,
        facet: Option<&str>,
        pagination: &Pagination,
    ) -> TrackTagsResult<Vec<TagTermCount>> {
        let mut target = aux_tracks_tag::table
            .select((aux_tracks_tag::term, sql::<diesel::sql_types::BigInt>("count(*) AS count")))
            .group_by(aux_tracks_tag::term)
            .order(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Facet Filtering
        target = match facet {
            Some(facet) => target.filter(aux_tracks_tag::facet.eq(facet)),
            None => target.filter(aux_tracks_tag::facet.is_null()),
        };

        if let Some(offset) = pagination.offset {
            target = target.offset(offset as i64);
        };
        if let Some(limit) = pagination.limit {
            target = target.limit(limit as i64);
        };

        // Pagination
        if let Some(offset) = pagination.offset {
            target = target.offset(offset as i64);
        };
        if let Some(limit) = pagination.limit {
            target = target.limit(limit as i64);
        };

        if let Some(collection_uid) = collection_uid {
            let target = target
                .inner_join(aux_tracks_resource::table.on(
                    aux_tracks_tag::track_id.eq(aux_tracks_resource::track_id)
                    .and(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))
            ));
            let rows = target.load::<(String, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagTermCount {
                    term: row.0, count: row.1 as usize
                });
            }

            Ok(result)
        } else {
            let rows = target.load::<(String, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagTermCount {
                    term: row.0, count: row.1 as usize
                });
            }

            Ok(result)
        }

    }

    fn all_tags(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
    ) -> TrackTagsResult<Vec<TagCount>> {
        let mut target = aux_tracks_tag::table
            .select((aux_tracks_tag::facet, aux_tracks_tag::term, sql::<diesel::sql_types::BigInt>("count(*) AS count")))
            .group_by(aux_tracks_tag::facet)
            .group_by(aux_tracks_tag::term)
            .order(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Pagination
        if let Some(offset) = pagination.offset {
            target = target.offset(offset as i64);
        };
        if let Some(limit) = pagination.limit {
            target = target.limit(limit as i64);
        };

        if let Some(collection_uid) = collection_uid {
            let target = target
                .inner_join(aux_tracks_resource::table.on(
                    aux_tracks_tag::track_id.eq(aux_tracks_resource::track_id)
                    .and(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))
            ));
            let rows = target.load::<(Option<String>, String, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagCount {
                    facet: row.0, term: row.1, count: row.2 as usize
                });
            }

            Ok(result)
        } else {
            let rows = target.load::<(Option<String>, String, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagCount {
                    facet: row.0, term: row.1, count: row.2 as usize
                });
            }

            Ok(result)
        }

    }

}