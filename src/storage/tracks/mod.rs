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

use self::models::*;

use self::schema::*;

use super::*;

use super::util::*;

use super::serde::{deserialize_with_format, serialize_with_format, SerializationFormat,
                   SerializedEntity};

use super::collections::{schema::collections_entity, CollectionRepository};

use diesel;
use diesel::dsl::*;
use diesel::prelude::*;

use failure;

use std::i64;

use usecases::{api::*,
               Collections,
               TrackTags,
               TrackTagsResult,
               Tracks,
               TracksResult};

use aoide_core::{audio::*,
                 domain::{collection::{CollectionBody, CollectionEntity},
                          entity::*,
                          metadata::*,
                          track::*}};

mod models;

mod schema;

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

    pub fn recreate_missing_collections(
        &self,
        collection_prototype: &CollectionBody,
    ) -> Result<Vec<CollectionEntity>, failure::Error> {
        let orphaned_collection_uids = aux_tracks_resource::table
            .select(aux_tracks_resource::collection_uid)
            .distinct()
            .filter(
                aux_tracks_resource::collection_uid
                    .ne_all(collections_entity::table.select(collections_entity::uid)),
            )
            .load::<String>(self.connection)?;
        let mut recreated_collections = Vec::with_capacity(orphaned_collection_uids.len());
        if !orphaned_collection_uids.is_empty() {
            let collection_repo = CollectionRepository::new(self.connection);
            for collection_uid in orphaned_collection_uids {
                info!("Recreating missing collection: {}", collection_uid);
                let collection_entity = CollectionEntity::new(
                    EntityHeader::with_uid(collection_uid),
                    collection_prototype.clone(),
                );
                collection_repo.insert_entity(&collection_entity)?;
                recreated_collections.push(collection_entity);
            }
        }
        Ok(recreated_collections)
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
        for track_ref in track_body.references.iter() {
            let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Track, &track_ref);
            let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
            query.execute(self.connection)?;
        }
        for actor in track_body.actors.iter() {
            for actor_ref in actor.references.iter() {
                let insertable =
                    InsertableTracksRef::bind(track_id, RefOrigin::TrackActor, &actor_ref);
                let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                query.execute(self.connection)?;
            }
        }
        if let Some(album) = track_body.album.as_ref() {
            for album_ref in album.references.iter() {
                let insertable = InsertableTracksRef::bind(track_id, RefOrigin::Album, &album_ref);
                let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                query.execute(self.connection)?;
            }
            for actor in album.actors.iter() {
                for actor_ref in actor.references.iter() {
                    let insertable =
                        InsertableTracksRef::bind(track_id, RefOrigin::AlbumActor, &actor_ref);
                    let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                    query.execute(self.connection)?;
                }
            }
        }
        if let Some(release) = track_body.release.as_ref() {
            for release_ref in release.references.iter() {
                let insertable =
                    InsertableTracksRef::bind(track_id, RefOrigin::Release, &release_ref);
                let query = diesel::replace_into(aux_tracks_ref::table).values(&insertable);
                query.execute(self.connection)?;
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

    fn after_entity_inserted(&self, entity: &TrackEntity) -> Result<StorageId, failure::Error> {
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

fn select_track_ids_matching_tag_filter<'a, DB>(
    tag_filter: TagFilter,
) -> (diesel::query_builder::BoxedSelectStatement<
    'a,
    diesel::sql_types::BigInt,
    aux_tracks_tag::table,
    DB,
>, Option<FilterModifier>)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = aux_tracks_tag::table
        .select(aux_tracks_tag::track_id)
        .into_boxed();

    // Filter tag facet
    if tag_filter.facet == TagFilter::no_facet() {
        select = select.filter(aux_tracks_tag::facet.is_null());
    } else if let Some(facet) = tag_filter.facet {
        select = select.filter(aux_tracks_tag::facet.eq(facet));
    }

    // Filter tag term
    if let Some(term_condition) = tag_filter.term_condition {
        let (either_eq_or_like, modifier) = match term_condition {
            // Equal comparison
            StringCondition::Matches(condition_params) => (
                EitherEqualOrLike::Equal(condition_params.value),
                condition_params.modifier,
            ),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCondition::StartsWith(condition_params) => (
                EitherEqualOrLike::Like(format!(
                    "{}%",
                    condition_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                condition_params.modifier,
            ),
            StringCondition::EndsWith(condition_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}",
                    condition_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                condition_params.modifier,
            ),
            StringCondition::Contains(condition_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}%",
                    condition_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                condition_params.modifier,
            ),
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => select.filter(aux_tracks_tag::term.eq(eq)),
                Some(ConditionModifier::Complement) => select.filter(aux_tracks_tag::term.ne(eq)),
            }
            EitherEqualOrLike::Like(like) => match modifier {
                None => select.filter(aux_tracks_tag::term.like(like).escape('\\')),
                Some(ConditionModifier::Complement) => select.filter(aux_tracks_tag::term.not_like(like).escape('\\')),
            }
        };
    }

    // Filter tag score
    if let Some(score_condition) = tag_filter.score_condition {
        select = match score_condition {
            ScoreCondition::LessThan(condition_params) => match condition_params.modifier {
                None => select.filter(aux_tracks_tag::score.lt(*condition_params.value)),
                Some(ConditionModifier::Complement) => select.filter(aux_tracks_tag::score.ge(*condition_params.value)),
            }
            ScoreCondition::GreaterThan(condition_params) => match condition_params.modifier {
                None => select.filter(aux_tracks_tag::score.gt(*condition_params.value)),
                Some(ConditionModifier::Complement) => select.filter(aux_tracks_tag::score.le(*condition_params.value)),
            }
            ScoreCondition::EqualTo(condition_params) => match condition_params.modifier {
                None => select.filter(aux_tracks_tag::score.eq(*condition_params.value)),
                Some(ConditionModifier::Complement) => select.filter(aux_tracks_tag::score.ne(*condition_params.value)),
            }
        };
    }

    (select, tag_filter.modifier)
}

impl<'a> EntityStorage for TrackRepository<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        let result = tracks_entity::table
            .select(tracks_entity::id)
            .filter(tracks_entity::uid.eq(uid.as_str()))
            .first::<StorageId>(self.connection)
            .optional()?;
        Ok(result)
    }
}

enum EitherEqualOrLike {
    Equal(String),
    Like(String),
}

impl<'a> Tracks for TrackRepository<'a> {
    fn create_entity(
        &self,
        body: TrackBody,
        format: SerializationFormat,
    ) -> TracksResult<TrackEntity> {
        let entity = TrackEntity::with_body(body);
        self.insert_entity(&entity, format)?;
        Ok(entity)
    }

    fn insert_entity(&self, entity: &TrackEntity, format: SerializationFormat) -> TracksResult<()> {
        {
            let entity_blob = serialize_with_format(entity, format)?;
            let insertable = InsertableTracksEntity::bind(entity.header(), format, &entity_blob);
            let query = diesel::insert_into(tracks_entity::table).values(&insertable);
            query.execute(self.connection)?;
        }
        self.after_entity_inserted(&entity)?;
        Ok(())
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

    fn replace_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        replacement_params: TrackReplacementParams,
        format: SerializationFormat,
    ) -> TracksResult<TrackReplacementReport> {
        let mut report = TrackReplacementReport::default();
        for replacement in replacement_params.replacements.into_iter() {
            let uri_filter = StringCondition::Matches(StringConditionParams {
                value: replacement.uri.clone(),
                modifier: None,
            });
            let locate_params = LocateParams { uri_filter };
            let located_entities =
                self.locate_entities(collection_uid, &Pagination::default(), locate_params)?;
            // Ambiguous?
            if located_entities.len() > 1 {
                assert!(collection_uid.is_none());
                warn!("Found multiple tracks with URI '{}' in different collections", replacement.uri);
                report.rejected.push(replacement.uri);
                continue;
            }
            if !replacement.track.is_valid() {
                warn!("Replacing track although it is not valid: {:?}", replacement.track);
                // ...ignore issues and continue
            }
            // Update?
            if let Some(serialized_entity) = located_entities.first() {
                let mut entity = deserialize_with_format::<TrackEntity>(serialized_entity)?;
                if entity.body() == &replacement.track {
                    debug!("Track '{}' is unchanged and does not need to be updated", entity.header().uid());
                    report.skipped.push(entity.into_header());
                    continue;
                }
                entity.replace_body(replacement.track);
                self.update_entity(&mut entity, format)?;
                report.updated.push(entity.into_header());
                continue;
            }
            // Create?
            match replacement_params.mode {
                ReplaceMode::UpdateOnly => {
                    info!("Track with URI '{}' does not exist and needs to be created", replacement.uri);
                    report.discarded.push(replacement.uri);
                    continue;
                }
                ReplaceMode::UpdateOrCreate => {
                    if let Some(collection_uid) = collection_uid {
                        // Check consistency to avoid unique constraint violations
                        // when inserting into the database.
                        match replacement.track.resource(collection_uid) {
                            Some(resource) => {
                                if resource.source.uri != replacement.uri {
                                    warn!("Mismatching track URI: expected = '{}', actual = '{}'", replacement.uri, resource.source.uri);
                                    report.rejected.push(replacement.uri);
                                    continue;
                                }
                            }
                            None => {
                                warn!("Track with URI '{}' does not belong to collection '{}'", replacement.uri,
                                    collection_uid);
                                report.rejected.push(replacement.uri);
                                continue;
                            }
                        }
                    }
                    let entity = self.create_entity(replacement.track, format)?;
                    report.created.push(entity.into_header())
                }
            };
        }
        Ok(report)
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
        // URI filter
        let (either_eq_or_like, modifier) = match locate_params.uri_filter {
            // Equal comparison
            StringCondition::Matches(condition_params) => (
                EitherEqualOrLike::Equal(condition_params.value),
                condition_params.modifier,
            ),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCondition::StartsWith(condition_params) => (
                EitherEqualOrLike::Like(format!(
                    "{}%",
                    condition_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                condition_params.modifier,
            ),
            StringCondition::EndsWith(condition_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}",
                    condition_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                condition_params.modifier,
            ),
            StringCondition::Contains(condition_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}%",
                    condition_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                condition_params.modifier,
            ),
        };

        // A subselect has proven to be much more efficient than
        // joining the aux_tracks_resource table!!
        let mut track_id_subselect = aux_tracks_resource::table
            .select(aux_tracks_resource::track_id)
            .into_boxed();
        if let Some(collection_uid) = collection_uid {
            track_id_subselect = track_id_subselect
                .filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()));
        };
        track_id_subselect = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => track_id_subselect.filter(aux_tracks_resource::source_uri.eq(eq)),
                Some(ConditionModifier::Complement) => {
                    track_id_subselect.filter(aux_tracks_resource::source_uri.ne(eq))
                }
            },
            EitherEqualOrLike::Like(like) => match modifier {
                None => track_id_subselect.filter(aux_tracks_resource::source_uri.like(like).escape('\\')),
                Some(ConditionModifier::Complement) => {
                    track_id_subselect.filter(aux_tracks_resource::source_uri.not_like(like).escape('\\'))
                }
            },
        };

        let mut target = tracks_entity::table
            .select(tracks_entity::all_columns)
            .filter(tracks_entity::id.eq_any(track_id_subselect))
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        let results: Vec<QueryableSerializedEntity> = target.load(self.connection)?;
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
        search_params: SearchParams,
    ) -> TracksResult<Vec<SerializedEntity>> {
        let mut target = tracks_entity::table
            .select(tracks_entity::all_columns)
            .left_outer_join(aux_tracks_resource::table)
            .left_outer_join(aux_tracks_overview::table)
            .left_outer_join(aux_tracks_summary::table)
            .left_outer_join(aux_tracks_music::table)
            .into_boxed();

        if let Some(phrase_filter) = search_params.phrase_filter {
            // Escape wildcard character with backslash (see below)
            let escaped_query = phrase_filter
                .query
                .replace('\\', "\\\\")
                .replace('%', "\\%");
            let escaped_and_tokenized = escaped_query
                .split_whitespace()
                .filter(|token| !token.is_empty());
            let escaped_and_tokenized_len = escaped_and_tokenized
                .clone()
                .fold(0, |len, token| len + token.len());
            // TODO: Use Rc<String> to avoid cloning strings?
            let like_expr = if escaped_and_tokenized_len > 0 {
                let mut like_expr = escaped_and_tokenized.fold(
                    String::with_capacity(1 + escaped_and_tokenized_len + 1), // leading/trailing '%'
                    |mut like_expr, part| {
                        // Prepend wildcard character before each part
                        like_expr.push('%');
                        like_expr.push_str(part);
                        like_expr
                    },
                );
                // Append final wildcard character after last part
                like_expr.push('%');
                like_expr
            } else {
                // unused
                String::new()
            };
            if !like_expr.is_empty() {
                // aux_track_resource (join)
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::Source)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_tracks_resource::source_uri_decoded
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Inverse) => target.or_filter(
                            aux_tracks_resource::source_uri_decoded
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::MediaType)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_tracks_resource::media_type
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Inverse) => target.or_filter(
                            aux_tracks_resource::media_type
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }

                // aux_track_overview (join)
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::TrackTitle)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_tracks_overview::track_title
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Inverse) => target.or_filter(
                            aux_tracks_overview::track_title
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::AlbumTitle)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_tracks_overview::album_title
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Inverse) => target.or_filter(
                            aux_tracks_overview::album_title
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }

                // aux_tracks_summary (join)
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::TrackArtist)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_tracks_summary::track_artist
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Inverse) => target.or_filter(
                            aux_tracks_summary::track_artist
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::AlbumArtist)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_tracks_summary::album_artist
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Inverse) => target.or_filter(
                            aux_tracks_summary::album_artist
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }

                // aux_track_comment (subselect)
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::Comments)
                {
                    let subselect = aux_tracks_comment::table
                        .select(aux_tracks_comment::track_id)
                        .filter(
                            aux_tracks_comment::text
                                .like(like_expr.clone())
                                .escape('\\'),
                        );
                    target = match phrase_filter.modifier {
                        None => target.or_filter(tracks_entity::id.eq_any(subselect)),
                        Some(FilterModifier::Inverse) => {
                            target.or_filter(tracks_entity::id.ne_all(subselect))
                        }
                    };
                }
            }
        }

        for tag_filter in search_params.tag_filters.into_iter() {
            let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(tag_filter);
            target = match filter_modifier {
                None => target.filter(tracks_entity::id.eq_any(subselect)),
                Some(FilterModifier::Inverse) => {
                    target.filter(tracks_entity::id.ne_all(subselect))
                }
            }
        }

        for numeric_filter in search_params.numeric_filters {
            target = match numeric_filter.field {
                NumericField::DurationMs => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_duration_ms.lt(condition_params.value))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_duration_ms.ge(condition_params.value))
                        }
                    }
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_duration_ms.gt(condition_params.value))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_duration_ms.le(condition_params.value))
                        }
                    }
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_duration_ms.eq(condition_params.value))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_duration_ms.ne(condition_params.value))
                        }
                    }
                }
                NumericField::SamplerateHz => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_samplerate_hz.lt(condition_params.value as i32))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_samplerate_hz.ge(condition_params.value as i32))
                        }
                    }
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_samplerate_hz.gt(condition_params.value as i32))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_samplerate_hz.le(condition_params.value as i32))
                        }
                    }
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_samplerate_hz.eq(condition_params.value as i32))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_samplerate_hz.ne(condition_params.value as i32))
                        }
                    }
                }
                NumericField::BitrateBps => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_bitrate_bps.lt(condition_params.value as i32))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_bitrate_bps.ge(condition_params.value as i32))
                        }
                    }
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_bitrate_bps.gt(condition_params.value as i32))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_bitrate_bps.le(condition_params.value as i32))
                        }
                    }
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_bitrate_bps.eq(condition_params.value as i32))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_bitrate_bps.ne(condition_params.value as i32))
                        }
                    }
                }
                NumericField::ChannelsCount => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_channels_count.lt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_channels_count.ge(condition_params.value as i16))
                        }
                    }
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_channels_count.gt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_channels_count.le(condition_params.value as i16))
                        }
                    }
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_resource::audio_channels_count.eq(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_resource::audio_channels_count.ne(condition_params.value as i16))
                        }
                    }
                }
                NumericField::TempoBpm => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::tempo_bpm.lt(condition_params.value))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::tempo_bpm.ge(condition_params.value))
                        }
                    },
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::tempo_bpm.gt(condition_params.value))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::tempo_bpm.le(condition_params.value))
                        }
                    },
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::tempo_bpm.eq(condition_params.value))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::tempo_bpm.ne(condition_params.value))
                        }
                    },
                }
                NumericField::KeysigCode => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::keysig_code.lt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::keysig_code.ge(condition_params.value as i16))
                        }
                    },
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::keysig_code.gt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::keysig_code.le(condition_params.value as i16))
                        }
                    },
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::keysig_code.eq(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::keysig_code.ne(condition_params.value as i16))
                        }
                    },
                }
                NumericField::TimesigNum => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::timesig_num.lt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::timesig_num.ge(condition_params.value as i16))
                        }
                    },
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::timesig_num.gt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::timesig_num.le(condition_params.value as i16))
                        }
                    },
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::timesig_num.eq(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::timesig_num.ne(condition_params.value as i16))
                        }
                    },
                }
                NumericField::TimesigDenom => match numeric_filter.condition {
                    NumericValueCondition::LessThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::timesig_denom.lt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::timesig_denom.ge(condition_params.value as i16))
                        }
                    },
                    NumericValueCondition::GreaterThan(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::timesig_denom.gt(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::timesig_denom.le(condition_params.value as i16))
                        }
                    },
                    NumericValueCondition::EqualTo(condition_params) => match condition_params.modifier {
                        None => {
                            target.filter(aux_tracks_music::timesig_denom.eq(condition_params.value as i16))
                        }
                        Some(ConditionModifier::Complement) => {
                            target.filter(aux_tracks_music::timesig_denom.ne(condition_params.value as i16))
                        }
                    },
                }
            };
        }

        // Collection filter
        if let Some(uid) = collection_uid {
            target = target.filter(aux_tracks_resource::collection_uid.eq(uid.as_str()));
        };

        for sort_order in search_params.ordering {
            let direction = sort_order
                .direction
                .unwrap_or_else(|| TrackSort::default_direction(sort_order.field));
            target =
                match sort_order.field {
                    field @ TrackSortField::InCollectionSince => {
                        if collection_uid.is_some() {
                            match direction {
                                SortDirection::Ascending => target
                                    .then_order_by(aux_tracks_resource::collection_since.asc()),
                                SortDirection::Descending => target
                                    .then_order_by(aux_tracks_resource::collection_since.desc()),
                            }
                        } else {
                            warn!("Cannot order by {:?} over multiple collections", field);
                            target
                        }
                    }
                    TrackSortField::LastRevisionedAt => match direction {
                        SortDirection::Ascending => {
                            target.then_order_by(tracks_entity::rev_timestamp.asc())
                        }
                        SortDirection::Descending => {
                            target.then_order_by(tracks_entity::rev_timestamp.desc())
                        }
                    },
                    TrackSortField::TrackTitle => match direction {
                        SortDirection::Ascending => {
                            target.then_order_by(aux_tracks_overview::track_title.asc())
                        }
                        SortDirection::Descending => {
                            target.then_order_by(aux_tracks_overview::track_title.desc())
                        }
                    },
                    TrackSortField::AlbumTitle => match direction {
                        SortDirection::Ascending => {
                            target.then_order_by(aux_tracks_overview::album_title.asc())
                        }
                        SortDirection::Descending => {
                            target.then_order_by(aux_tracks_overview::album_title.desc())
                        }
                    },
                    TrackSortField::TrackArtist => match direction {
                        SortDirection::Ascending => {
                            target.then_order_by(aux_tracks_summary::track_artist.asc())
                        }
                        SortDirection::Descending => {
                            target.then_order_by(aux_tracks_summary::track_artist.desc())
                        }
                    },
                    TrackSortField::AlbumArtist => match direction {
                        SortDirection::Ascending => {
                            target.then_order_by(aux_tracks_summary::album_artist.asc())
                        }
                        SortDirection::Descending => {
                            target.then_order_by(aux_tracks_summary::album_artist.desc())
                        }
                    },
                }
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let results = target.load::<QueryableSerializedEntity>(self.connection)?;

        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    fn field_counts(
        &self,
        collection_uid: Option<&EntityUid>,
        field: CountableStringField,
    ) -> TracksResult<StringFieldCounts> {
        let track_id_subselect = collection_uid.map(|collection_uid| {
            aux_tracks_resource::table
                .select(aux_tracks_resource::track_id)
                .filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))
        });
        let rows = match field {
            CountableStringField::MediaType => {
                let mut target = aux_tracks_resource::table
                    .select((
                        aux_tracks_resource::media_type,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_tracks_resource::media_type)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_tracks_resource::media_type)
                    .into_boxed();
                if let Some(collection_uid) = collection_uid {
                    target =
                        target.filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()));
                }
                let rows = target.load::<(String, i64)>(self.connection)?;
                // TODO: Remove this transformation and select media_type
                // as a nullable column?!
                rows.into_iter()
                    .map(|(media_type, count)| (Some(media_type), count))
                    .collect()
            }
            CountableStringField::TrackTitle => {
                let mut target = aux_tracks_overview::table
                    .select((
                        aux_tracks_overview::track_title,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_tracks_overview::track_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_tracks_overview::track_title)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target =
                        target.filter(aux_tracks_overview::track_id.eq_any(track_id_subselect));
                }
                target.load::<(Option<String>, i64)>(self.connection)?
            }
            CountableStringField::AlbumTitle => {
                let mut target = aux_tracks_overview::table
                    .select((
                        aux_tracks_overview::album_title,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_tracks_overview::album_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_tracks_overview::album_title)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target =
                        target.filter(aux_tracks_overview::track_id.eq_any(track_id_subselect));
                }
                target.load::<(Option<String>, i64)>(self.connection)?
            }
            CountableStringField::TrackArtist => {
                let mut target = aux_tracks_summary::table
                    .select((
                        aux_tracks_summary::track_artist,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_tracks_summary::track_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_tracks_summary::track_artist)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_tracks_summary::track_id.eq_any(track_id_subselect));
                }
                target.load::<(Option<String>, i64)>(self.connection)?
            }
            CountableStringField::AlbumArtist => {
                let mut target = aux_tracks_summary::table
                    .select((
                        aux_tracks_summary::album_artist,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_tracks_summary::album_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_tracks_summary::album_artist)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_tracks_summary::track_id.eq_any(track_id_subselect));
                }
                target.load::<(Option<String>, i64)>(self.connection)?
            }
        };
        let mut counts = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            let value = row.0;
            debug_assert!(row.1 > 0);
            let count = row.1 as usize;
            counts.push(StringCount { value, count });
        }
        Ok(StringFieldCounts { field, counts })
    }

    fn resource_statistics(
        &self,
        collection_uid: Option<&EntityUid>,
    ) -> TracksResult<ResourceStats> {
        let total_count = {
            let mut target = aux_tracks_resource::table
                .select(diesel::dsl::count_star())
                .into_boxed();
            // Collection filtering
            target = match collection_uid {
                Some(uid) => target.filter(aux_tracks_resource::collection_uid.eq(uid.as_str())),
                None => target,
            };
            target.first::<i64>(self.connection)? as usize
        };

        let total_duration_ms = {
            let mut target = aux_tracks_resource::table
                .select(diesel::dsl::sum(aux_tracks_resource::audio_duration_ms))
                .into_boxed();
            // Collection filtering
            target = match collection_uid {
                Some(uid) => target.filter(aux_tracks_resource::collection_uid.eq(uid.as_str())),
                None => target,
            };
            target.first::<Option<f64>>(self.connection)?
        };
        let total_duration = total_duration_ms
            .map(|ms| Duration { ms })
            .unwrap_or(Duration::EMPTY);

        let media_types = {
            let mut target = aux_tracks_resource::table
                .select((
                    aux_tracks_resource::media_type,
                    sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                ))
                .group_by(aux_tracks_resource::media_type)
                .into_boxed();
            // Collection filtering
            target = match collection_uid {
                Some(uid) => target.filter(aux_tracks_resource::collection_uid.eq(uid.as_str())),
                None => target,
            };
            let rows = target.load::<(String, i64)>(self.connection)?;
            let mut media_types: Vec<MediaTypeStats> = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                media_types.push(MediaTypeStats {
                    media_type: row.0,
                    count: row.1 as usize,
                });
            }
            media_types
        };

        Ok(ResourceStats {
            count: total_count,
            duration: total_duration,
            media_types,
        })
    }
}

impl<'a> TrackTags for TrackRepository<'a> {
    fn all_tags_facets(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: &Pagination,
    ) -> TrackTagsResult<Vec<TagFacetCount>> {
        let mut target = aux_tracks_tag::table
            .select((
                aux_tracks_tag::facet,
                sql::<diesel::sql_types::BigInt>("count(*) AS count"),
            ))
            .group_by(aux_tracks_tag::facet)
            .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Facet Filtering
        target = match facets {
            Some(facets) => {
                if facets.is_empty() {
                    target.filter(aux_tracks_tag::facet.is_null())
                } else {
                    let filtered = target.filter(aux_tracks_tag::facet.eq_any(facets));
                    if facets.iter().any(|facet| facet.is_empty()) {
                        // Empty facets are interpreted as null, just like an empty vector
                        filtered.or_filter(aux_tracks_tag::facet.is_null())
                    } else {
                        filtered
                    }
                }
            }
            None => target,
        };

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_tracks_resource::table.select(aux_tracks_resource::track_id).filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()));
            target = target.filter(aux_tracks_tag::track_id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(Option<String>, i64)>(self.connection)?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            result.push(TagFacetCount {
                facet: row.0,
                count: row.1 as usize,
            });
        }

        Ok(result)
    }

    fn all_tags(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: &Pagination,
    ) -> TrackTagsResult<Vec<MultiTag>> {
        let mut target = aux_tracks_tag::table
            .select((
                aux_tracks_tag::facet,
                aux_tracks_tag::term,
                sql::<diesel::sql_types::Double>("AVG(score) AS score"),
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS count"),
            ))
            .group_by(aux_tracks_tag::facet)
            .group_by(aux_tracks_tag::term)
            .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Facet Filtering
        target = match facets {
            Some(facets) => {
                if facets.is_empty() {
                    target.filter(aux_tracks_tag::facet.is_null())
                } else {
                    let filtered = target.filter(aux_tracks_tag::facet.eq_any(facets));
                    if facets.iter().any(|facet| facet.is_empty()) {
                        // Empty facets are interpreted as null, just like an empty vector
                        filtered.or_filter(aux_tracks_tag::facet.is_null())
                    } else {
                        filtered
                    }
                }
            }
            None => target,
        };

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_tracks_resource::table.select(aux_tracks_resource::track_id).filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()));
            target = target.filter(aux_tracks_tag::track_id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(Option<String>, String, f64, i64)>(self.connection)?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
            result.push(MultiTag {
                tag: Tag {
                    facet: row.0,
                    term: row.1,
                    score: Score(row.2),
                },
                count: row.3 as usize,
            });
        }

        Ok(result)
    }
}
