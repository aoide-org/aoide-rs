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

use super::collections::CollectionRepository;
use super::collections::schema::collections_entity;

use diesel;
use diesel::dsl::*;
use diesel::prelude::*;

use failure;

use std::i64;

use super::serde::{deserialize_with_format, serialize_with_format, SerializationFormat,
                   SerializedEntity};
use super::*;

use usecases::request::{FilterModifier, LocateParams, PhraseFilterField, ReplaceMode,
                        ReplaceParams, ScoreFilter, SearchParams, StringFilter,
                        StringFilterParams, TagFilter};
use usecases::result::Pagination;
use usecases::{Collections, TrackEntityReplacement, TrackTags, TrackTagsResult, Tracks,
               TracksResult};

use aoide_core::domain::collection::*;
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
) -> diesel::query_builder::BoxedSelectStatement<
    'a,
    diesel::sql_types::BigInt,
    aux_tracks_tag::table,
    DB,
>
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
    if let Some(term_filter) = tag_filter.term_filter {
        let (either_eq_or_like, modifier) = match term_filter {
            // Equal comparison
            StringFilter::Matches(filter_params) => (
                EitherEqualOrLike::Equal(filter_params.value),
                filter_params.modifier,
            ),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringFilter::StartsWith(filter_params) => (
                EitherEqualOrLike::Like(format!(
                    "{}%",
                    filter_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                filter_params.modifier,
            ),
            StringFilter::EndsWith(filter_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}",
                    filter_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                filter_params.modifier,
            ),
            StringFilter::Contains(filter_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}%",
                    filter_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                filter_params.modifier,
            ),
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => select.filter(aux_tracks_tag::term.eq(eq)),
                Some(FilterModifier::Inverse) => select.filter(aux_tracks_tag::term.ne(eq)),
            },
            EitherEqualOrLike::Like(like) => match modifier {
                None => select.filter(aux_tracks_tag::term.like(like).escape('\\')),
                Some(FilterModifier::Inverse) => {
                    select.filter(aux_tracks_tag::term.not_like(like).escape('\\'))
                }
            },
        };
    }

    // Filter tag score
    if let Some(score_filter) = tag_filter.score_filter {
        select = match score_filter {
            ScoreFilter::LessThan(filter_params) => match filter_params.modifier {
                None => select.filter(aux_tracks_tag::score.lt(*filter_params.value)),
                Some(FilterModifier::Inverse) => {
                    select.filter(aux_tracks_tag::score.ge(*filter_params.value))
                }
            },
            ScoreFilter::GreaterThan(filter_params) => match filter_params.modifier {
                None => select.filter(aux_tracks_tag::score.gt(*filter_params.value)),
                Some(FilterModifier::Inverse) => {
                    select.filter(aux_tracks_tag::score.le(*filter_params.value))
                }
            },
            ScoreFilter::EqualTo(filter_params) => match filter_params.modifier {
                None => select.filter(aux_tracks_tag::score.eq(*filter_params.value)),
                Some(FilterModifier::Inverse) => {
                    select.filter(aux_tracks_tag::score.ne(*filter_params.value))
                }
            },
        };
    }

    select
}

fn apply_pagination<'a, ST, QS, DB>(
    source: diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>,
    pagination: &Pagination,
) -> diesel::query_builder::BoxedSelectStatement<'a, ST, QS, DB>
where
    QS: diesel::query_source::QuerySource,
    DB: diesel::backend::Backend + diesel::sql_types::HasSqlType<ST> + 'a,
{
    let mut target = source;
    if let Some(offset) = pagination.offset {
        target = target.offset(offset as i64);
    };
    if let Some(limit) = pagination.limit {
        target = target.limit(limit as i64);
    };
    target
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

    fn replace_entity(
        &self,
        collection_uid: Option<&EntityUid>,
        replace_params: ReplaceParams,
        format: SerializationFormat,
    ) -> TracksResult<TrackEntityReplacement> {
        let uri_filter = StringFilter::Matches(StringFilterParams {
            value: replace_params.uri.clone(),
            modifier: None,
        });
        let locate_params = LocateParams { uri_filter };
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

        // URI filter
        let (either_eq_or_like, modifier) = match locate_params.uri_filter {
            // Equal comparison
            StringFilter::Matches(filter_params) => (
                EitherEqualOrLike::Equal(filter_params.value),
                filter_params.modifier,
            ),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringFilter::StartsWith(filter_params) => (
                EitherEqualOrLike::Like(format!(
                    "{}%",
                    filter_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                filter_params.modifier,
            ),
            StringFilter::EndsWith(filter_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}",
                    filter_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                filter_params.modifier,
            ),
            StringFilter::Contains(filter_params) => (
                EitherEqualOrLike::Like(format!(
                    "%{}%",
                    filter_params
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                filter_params.modifier,
            ),
        };
        target = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => target.filter(aux_tracks_resource::source_uri.eq(eq)),
                Some(FilterModifier::Inverse) => {
                    target.filter(aux_tracks_resource::source_uri.ne(eq))
                }
            },
            EitherEqualOrLike::Like(like) => match modifier {
                None => target.filter(aux_tracks_resource::source_uri.like(like).escape('\\')),
                Some(FilterModifier::Inverse) => {
                    target.filter(aux_tracks_resource::source_uri.not_like(like).escape('\\'))
                }
            },
        };

        // Collection filter & ordering
        target = match collection_uid {
            Some(collection_uid) => target
                .filter(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))
                .order(aux_tracks_resource::collection_since.desc()), // recently added to collection
            None => target.order(tracks_entity::rev_timestamp.desc()), // recently modified
        };

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
        // Escape wildcard character with backslash (see below)
        let escaped_tokens = search_params.phrase_filter.as_ref().map(|phrase_filter| {
            phrase_filter
                .phrase
                .replace('\\', "\\\\")
                .replace('%', "\\%")
        });
        let escaped_and_tokenized = escaped_tokens
            .as_ref()
            .map(|tokens| tokens.split_whitespace().filter(|token| !token.is_empty()));
        let escaped_and_tokenized_len = escaped_and_tokenized
            .as_ref()
            .map(|tokenized| tokenized.clone().fold(0, |len, token| len + token.len()))
            .unwrap_or(0);
        // TODO: if/else arms are incompatible due to joining tables?
        let results = if escaped_and_tokenized_len == 0 {
            // Select all (without joining)
            let mut target = tracks_entity::table
                .select(tracks_entity::all_columns)
                .left_outer_join(aux_tracks_resource::table)
                .into_boxed();

            // Filter tags 1st level: Conjunction
            // TODO: Extract into a function (https://github.com/diesel-rs/diesel/issues/546)
            for tag_filters in search_params.tag_filters.into_iter() {
                // Filter tags 2nd level: Disjunction
                for (index, tag_filter) in tag_filters.into_iter().enumerate() {
                    let sub_query = select_track_ids_matching_tag_filter(tag_filter);
                    target = match index {
                        0 => target.filter(tracks_entity::id.eq_any(sub_query)),
                        _ => target.or_filter(tracks_entity::id.eq_any(sub_query)),
                    };
                }
            }

            // Collection filter & ordering
            // TODO: Extract into a function (https://github.com/diesel-rs/diesel/issues/546)
            target = match collection_uid {
                Some(uid) => target
                    .filter(aux_tracks_resource::collection_uid.eq(uid.as_str()))
                    .order(aux_tracks_resource::collection_since.desc()), // recently added to collection
                None => target.order(tracks_entity::rev_timestamp.desc()), // recently modified
            };

            // Pagination
            target = apply_pagination(target, pagination);

            target.load::<QueryableSerializedEntity>(self.connection)?
        } else {
            debug_assert!(escaped_and_tokenized_len > 0);
            let mut like_expr = escaped_and_tokenized.unwrap().fold(
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

            // TODO: Avoid unneeded joins according to the values in
            // search_params.phrase_filter.fields (see below)
            let mut target = tracks_entity::table
                .select(tracks_entity::all_columns)
                .left_outer_join(aux_tracks_resource::table)
                .left_outer_join(aux_tracks_overview::table)
                .left_outer_join(aux_tracks_summary::table)
                .into_boxed();

            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::Source)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        aux_tracks_resource::source_uri
                            .like(&like_expr)
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        aux_tracks_resource::source_uri
                            .not_like(&like_expr)
                            .escape('\\'),
                    ),
                };
            }
            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::Grouping)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        aux_tracks_overview::grouping
                            .like(&like_expr)
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        aux_tracks_overview::grouping
                            .not_like(&like_expr)
                            .escape('\\'),
                    ),
                };
            }
            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::TrackTitle)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        aux_tracks_overview::track_title
                            .like(&like_expr)
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        aux_tracks_overview::track_title
                            .not_like(&like_expr)
                            .escape('\\'),
                    ),
                };
            }
            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::AlbumTitle)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        aux_tracks_overview::album_title
                            .like(&like_expr)
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        aux_tracks_overview::album_title
                            .not_like(&like_expr)
                            .escape('\\'),
                    ),
                };
            }
            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::TrackArtist)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        aux_tracks_summary::track_artist
                            .like(&like_expr)
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        aux_tracks_summary::track_artist
                            .not_like(&like_expr)
                            .escape('\\'),
                    ),
                };
            }
            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::AlbumArtist)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        aux_tracks_summary::album_artist
                            .like(&like_expr)
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        aux_tracks_summary::album_artist
                            .not_like(&like_expr)
                            .escape('\\'),
                    ),
                };
            }
            if search_params
                .phrase_filter
                .as_ref()
                .unwrap()
                .fields
                .is_empty()
                || search_params
                    .phrase_filter
                    .as_ref()
                    .unwrap()
                    .fields
                    .iter()
                    .any(|target| *target == PhraseFilterField::Comments)
            {
                target = match search_params.phrase_filter.as_ref().unwrap().modifier {
                    None => target.or_filter(
                        tracks_entity::id.eq_any(
                            aux_tracks_comment::table
                                .select(aux_tracks_comment::track_id)
                                .filter(aux_tracks_comment::text.like(&like_expr).escape('\\')),
                        ),
                    ),
                    Some(FilterModifier::Inverse) => target.or_filter(
                        tracks_entity::id.ne_all(
                            aux_tracks_comment::table
                                .select(aux_tracks_comment::track_id)
                                .filter(aux_tracks_comment::text.like(&like_expr).escape('\\')),
                        ),
                    ),
                };
            }

            // Filter tags 1st level: Conjunction
            // TODO: Extract into a function (https://github.com/diesel-rs/diesel/issues/546)
            for tag_filters in search_params.tag_filters.into_iter() {
                // Filter tags 2nd level: Disjunction
                for (index, tag_filter) in tag_filters.into_iter().enumerate() {
                    let sub_query = select_track_ids_matching_tag_filter(tag_filter);
                    target = match index {
                        0 => target.filter(tracks_entity::id.eq_any(sub_query)),
                        _ => target.or_filter(tracks_entity::id.eq_any(sub_query)),
                    };
                }
            }

            // Collection filter & ordering
            // TODO: Extract into a function (https://github.com/diesel-rs/diesel/issues/546)
            target = match collection_uid {
                Some(uid) => target
                    .filter(aux_tracks_resource::collection_uid.eq(uid.as_str()))
                    .order(aux_tracks_resource::collection_since.desc()), // recently added to collection
                None => target.order(tracks_entity::rev_timestamp.desc()), // recently modified
            };

            // Pagination
            target = apply_pagination(target, pagination);

            target.load::<QueryableSerializedEntity>(self.connection)?
        };
        Ok(results.into_iter().map(|r| r.into()).collect())
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

        // Pagination
        target = apply_pagination(target, pagination);

        if let Some(collection_uid) = collection_uid {
            let target = target.inner_join(
                aux_tracks_resource::table.on(aux_tracks_tag::track_id
                    .eq(aux_tracks_resource::track_id)
                    .and(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))),
            );
            let rows = target.load::<(Option<String>, i64)>(self.connection)?;
            let mut result = Vec::with_capacity(rows.len());
            for row in rows.into_iter() {
                result.push(TagFacetCount {
                    facet: row.0,
                    count: row.1 as usize,
                });
            }

            Ok(result)
        } else {
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

        // Pagination
        target = apply_pagination(target, pagination);

        if let Some(collection_uid) = collection_uid {
            let target = target.inner_join(
                aux_tracks_resource::table.on(aux_tracks_tag::track_id
                    .eq(aux_tracks_resource::track_id)
                    .and(aux_tracks_resource::collection_uid.eq(collection_uid.as_str()))),
            );
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
        } else {
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
