// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

mod models;
mod schema;
mod search;
pub mod util;

use self::{
    models::*,
    schema::*,
    search::{TrackSearchBoxedExpressionBuilder, TrackSearchQueryTransform},
    util::TrackRepositoryHelper,
};

use crate::{
    api::{
        collection::CollectionTrackStats,
        serde::{serialize_with_format, SerializationFormat, SerializedEntity},
        track::*,
        *,
    },
    storage::util::*,
};

use diesel::dsl::*;

use validator::Validate;

///////////////////////////////////////////////////////////////////////
// TrackRepository
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct TrackRepository<'a> {
    connection: &'a diesel::SqliteConnection,
    helper: TrackRepositoryHelper<'a>,
}

impl<'a> TrackRepository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self {
            connection,
            helper: TrackRepositoryHelper::new(connection),
        }
    }
}

fn select_track_ids_matching_tag_filter<'a, DB>(
    tag_filter: &'a TagFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        aux_track_tag::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = aux_track_tag::table
        .select(aux_track_tag::track_id)
        .into_boxed();

    // Filter facet(s)
    if let Some(ref facets) = tag_filter.facets {
        if facets.is_empty() {
            // unfaceted tags without a facet
            select = select.filter(aux_track_tag::facet_id.is_null());
        } else {
            // tags with any of the given facets
            let subselect = aux_tag_facet::table
                .select(aux_tag_facet::id)
                .filter(aux_tag_facet::facet.eq_any(facets));
            select = select.filter(aux_track_tag::facet_id.eq_any(subselect.nullable()));
        }
    }

    // Filter labels
    if let Some(ref label) = tag_filter.label {
        let (cmp, val, dir) = label.into();
        let either_eq_or_like = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => EitherEqualOrLike::Equal(val.to_owned()),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCompare::StartsWith => EitherEqualOrLike::Like(format!(
                "{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::EndsWith => EitherEqualOrLike::Like(format!(
                "%{}",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Contains => EitherEqualOrLike::Like(format!(
                "%{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Matches => {
                EitherEqualOrLike::Like(val.replace('\\', "\\\\").replace('%', "\\%"))
            }
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => {
                if dir {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.eq(eq));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                } else {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.ne(eq));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                }
            }
            EitherEqualOrLike::Like(like) => {
                if dir {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.like(like).escape('\\'));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                } else {
                    let subselect = aux_tag_label::table
                        .select(aux_tag_label::id)
                        .filter(aux_tag_label::label.not_like(like).escape('\\'));
                    select.filter(aux_track_tag::label_id.eq_any(subselect.nullable()))
                }
            }
        };
    }

    // Filter tag score
    if let Some(score) = tag_filter.score {
        select = match score {
            NumericPredicate::LessThan(value) => select.filter(aux_track_tag::score.lt(value)),
            NumericPredicate::GreaterOrEqual(value) => {
                select.filter(aux_track_tag::score.ge(value))
            }
            NumericPredicate::GreaterThan(value) => select.filter(aux_track_tag::score.gt(value)),
            NumericPredicate::LessOrEqual(value) => select.filter(aux_track_tag::score.le(value)),
            NumericPredicate::Equal(value) => {
                if let Some(value) = value {
                    select.filter(aux_track_tag::score.eq(value))
                } else {
                    select.filter(aux_track_tag::score.is_null())
                }
            }
            NumericPredicate::NotEqual(value) => {
                if let Some(value) = value {
                    select.filter(aux_track_tag::score.ne(value))
                } else {
                    select.filter(aux_track_tag::score.is_not_null())
                }
            }
        };
    }

    (select, tag_filter.modifier)
}

fn select_track_ids_matching_marker_filter<'a, DB>(
    marker_filter: &'a MarkerFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        aux_track_marker::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = aux_track_marker::table
        .select(aux_track_marker::track_id)
        .into_boxed();

    // Filter labels
    if let Some(ref label) = marker_filter.label {
        let (cmp, val, dir) = label.into();
        let either_eq_or_like = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => EitherEqualOrLike::Equal(val.to_owned()),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCompare::StartsWith => EitherEqualOrLike::Like(format!(
                "{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::EndsWith => EitherEqualOrLike::Like(format!(
                "%{}",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Contains => EitherEqualOrLike::Like(format!(
                "%{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Matches => {
                EitherEqualOrLike::Like(val.replace('\\', "\\\\").replace('%', "\\%"))
            }
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => {
                if dir {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.eq(eq));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                } else {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.ne(eq));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                }
            }
            EitherEqualOrLike::Like(like) => {
                if dir {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.like(like).escape('\\'));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                } else {
                    let subselect = aux_marker_label::table
                        .select(aux_marker_label::id)
                        .filter(aux_marker_label::label.not_like(like).escape('\\'));
                    select.filter(aux_track_marker::label_id.eq_any(subselect))
                }
            }
        };
    }

    (select, marker_filter.modifier)
}

enum EitherEqualOrLike {
    Equal(String),
    Like(String),
}

impl<'a> Tracks for TrackRepository<'a> {
    fn create_entity(&self, body: Track, format: SerializationFormat) -> TracksResult<TrackEntity> {
        let entity = TrackEntity::new(EntityHeader::initial(), body);
        self.insert_entity(&entity, format)?;
        Ok(entity)
    }

    fn insert_entity(&self, entity: &TrackEntity, format: SerializationFormat) -> TracksResult<()> {
        {
            let entity_blob = serialize_with_format(entity, format)?;
            let insertable = InsertableTracksEntity::bind(entity.header(), format, &entity_blob);
            let query = diesel::insert_into(tbl_track::table).values(&insertable);
            query.execute(self.connection)?;
        }
        self.helper.after_entity_inserted(&entity)?;
        Ok(())
    }

    fn update_entity(
        &self,
        entity: TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<(EntityRevision, Option<EntityRevision>)> {
        let prev_revision = *entity.header().revision();
        let next_revision = prev_revision.next();
        {
            let uid = *entity.header().uid();
            let updated_entity = entity.replace_header_revision(next_revision);
            let entity_blob = serialize_with_format(&updated_entity, format)?;
            {
                let updatable = UpdatableTracksEntity::bind(&next_revision, format, &entity_blob);
                let target = tbl_track::table.filter(
                    tbl_track::uid
                        .eq(uid.as_ref())
                        .and(tbl_track::rev_no.eq(prev_revision.ordinal() as i64))
                        .and(tbl_track::rev_ts.eq((prev_revision.instant().0).0)),
                );
                let storage_id = self.helper.before_entity_updated_or_removed(&uid)?;
                let query = diesel::update(target).set(&updatable);
                let rows_affected: usize = query.execute(self.connection)?;
                debug_assert!(rows_affected <= 1);
                if rows_affected < 1 {
                    return Ok((prev_revision, None));
                }
                self.helper
                    .after_entity_updated(storage_id, &updated_entity.body())?;
            }
        }
        Ok((prev_revision, Some(next_revision)))
    }

    fn replace_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        replace_params: ReplaceTracksParams,
        format: SerializationFormat,
    ) -> TracksResult<ReplacedTracks> {
        let mut results = ReplacedTracks::default();
        for replacement in replace_params.replacements {
            let uri = StringPredicate::Equals(replacement.uri.clone());
            let locate_params = LocateTracksParams { uri };
            let located_entities =
                self.locate_entities(collection_uid, Pagination::default(), locate_params)?;
            let deserialized_entities: Vec<TrackEntity> = located_entities.iter().fold(
                Vec::with_capacity(located_entities.len()),
                |mut acc, item| {
                    match item.deserialize() {
                        Ok(deserialized) => {
                            acc.push(deserialized);
                        }
                        Err(e) => log::warn!("Failed to deserialize track entity: {}", e),
                    }
                    acc
                },
            );
            if deserialized_entities.len() < located_entities.len() {
                log::warn!(
                    "Failed to deserialize {} track(s) with URI '{}'",
                    located_entities.len() - deserialized_entities.len(),
                    replacement.uri
                );
                results.rejected.push(replacement.uri);
                continue;
            }
            // Ambiguous?
            if deserialized_entities.len() > 1 {
                log::warn!("Found multiple tracks with URI '{}'", replacement.uri);
                results.rejected.push(replacement.uri);
                continue;
            }
            // Valid?
            if let Err(validation_errors) = replacement.track.validate() {
                debug_assert!(!validation_errors.is_empty());
                log::warn!("Validation errors: {}", validation_errors);
                log::warn!(
                    "Accepting replacement track with validation errors: {:?}",
                    replacement.track
                );
                // ...ignore validation errors and continue
            }
            // Update?
            if let Some(entity) = deserialized_entities.into_iter().next() {
                let uid = *entity.header().uid();
                if entity.body() == &replacement.track {
                    log::debug!(
                        "Track '{}' is unchanged and does not need to be updated",
                        uid
                    );
                    results.skipped.push(*entity.header());
                    continue;
                }
                let replaced_entity = entity.replace_body(replacement.track);
                match self.update_entity(replaced_entity, format)? {
                    (_, None) => {
                        log::warn!(
                            "Failed to update track '{}' due to internal race condition",
                            uid
                        );
                        results.rejected.push(replacement.uri);
                    }
                    (_, Some(next_revision)) => {
                        let header = EntityHeader::new(uid, next_revision);
                        results.updated.push(header);
                    }
                };
            } else {
                // Create?
                match replace_params.mode {
                    ReplaceMode::UpdateOnly => {
                        log::info!(
                            "Track with URI '{}' does not exist and needs to be created",
                            replacement.uri
                        );
                        results.discarded.push(replacement.uri);
                        continue;
                    }
                    ReplaceMode::UpdateOrCreate => {
                        // Create!
                        let entity = self.create_entity(replacement.track, format)?;
                        results.created.push(*entity.header())
                    }
                };
            }
        }
        Ok(results)
    }

    fn delete_entity(&self, uid: &EntityUid) -> TracksResult<Option<()>> {
        let target = tbl_track::table.filter(tbl_track::uid.eq(uid.as_ref()));
        let query = diesel::delete(target);
        self.helper.before_entity_updated_or_removed(uid)?;
        let rows_affected: usize = query.execute(self.connection)?;
        debug_assert!(rows_affected <= 1);
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>> {
        tbl_track::table
            .filter(tbl_track::uid.eq(uid.as_ref()))
            .first::<QueryableSerializedEntity>(self.connection)
            .optional()
            .map(|o| o.map(Into::into))
            .map_err(Into::into)
    }

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        locate_params: LocateTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>> {
        // URI filter
        let (cmp, val, dir) = (&locate_params.uri).into();
        let either_eq_or_like = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => EitherEqualOrLike::Equal(val.to_owned()),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringCompare::StartsWith => EitherEqualOrLike::Like(format!(
                "{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::EndsWith => EitherEqualOrLike::Like(format!(
                "%{}",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Contains => EitherEqualOrLike::Like(format!(
                "%{}%",
                val.replace('\\', "\\\\").replace('%', "\\%")
            )),
            StringCompare::Matches => {
                EitherEqualOrLike::Like(val.replace('\\', "\\\\").replace('%', "\\%"))
            }
        };

        let mut target = tbl_track::table
            .select(tbl_track::all_columns)
            .order_by(tbl_track::id) // preserve relative order of results
            .into_boxed();

        // A subselect has proven to be much more efficient than
        // joining the aux_track_source table for filtering by URI!
        let mut track_id_subselect = aux_track_source::table
            .select(aux_track_source::track_id)
            .into_boxed();
        track_id_subselect = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => {
                if dir {
                    track_id_subselect.filter(aux_track_source::uri.eq(eq))
                } else {
                    track_id_subselect.filter(aux_track_source::uri.ne(eq))
                }
            }
            EitherEqualOrLike::Like(like) => {
                if dir {
                    track_id_subselect.filter(aux_track_source::uri.like(like).escape('\\'))
                } else {
                    track_id_subselect.filter(aux_track_source::uri.not_like(like).escape('\\'))
                }
            }
        };
        target = if dir {
            target.filter(tbl_track::id.eq_any(track_id_subselect))
        } else {
            target.filter(tbl_track::id.ne_all(track_id_subselect))
        };

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(tbl_track::id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableSerializedEntity>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        search_params: SearchTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>> {
        // TODO: Joins are very expensive and should only be used
        // when the results need to be ordered. For filtering
        // subselects have proven to be much more efficient.
        //
        // In general queries with joins are not suitable to be
        // executed efficiently as batch operations. Since search
        // operations are expected to be executed standalone the
        // joins are acceptable in this case.
        let mut target = tbl_track::table
            .select(tbl_track::all_columns)
            .distinct()
            .inner_join(aux_track_brief::table)
            .left_outer_join(aux_track_source::table)
            .left_outer_join(aux_track_collection::table)
            .into_boxed();

        if let Some(ref filter) = search_params.filter {
            target = target.filter(filter.build_expression(collection_uid));
        }

        // Collection filter
        if let Some(uid) = collection_uid {
            target = target.filter(aux_track_collection::collection_uid.eq(uid.as_ref()));
        };

        for sort_order in &search_params.ordering {
            target = sort_order.apply_to_query(target, collection_uid);
        }
        // Finally order by PK to preserve the relative order of results
        // even if no sorting was requested.
        target = target.then_order_by(tbl_track::id);

        // Pagination
        target = apply_pagination(target, pagination);

        let res = target.load::<QueryableSerializedEntity>(self.connection)?;
        Ok(res.into_iter().map(Into::into).collect())
    }

    fn list_field_strings(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: Pagination,
    ) -> TracksResult<FieldStrings> {
        let track_id_subselect = collection_uid.map(|collection_uid| {
            aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()))
        });
        let rows = match field {
            StringField::SourceUri => {
                let mut target = aux_track_source::table
                    .select((
                        aux_track_source::uri_decoded.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_source::uri_decoded)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_source::uri_decoded)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_source::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::ContentType => {
                let mut target = aux_track_source::table
                    .select((
                        aux_track_source::content_type.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_source::content_type)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_source::content_type)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_source::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackTitle => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::track_title.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::track_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::track_title)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackArtist => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::track_artist.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::track_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::track_artist)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackComposer => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::track_composer.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::track_composer)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::track_composer)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::AlbumTitle => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::album_title.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::album_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::album_title)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::AlbumArtist => {
                let mut target = aux_track_brief::table
                    .select((
                        aux_track_brief::album_artist.nullable(),
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_brief::album_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_brief::album_artist)
                    .into_boxed();

                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
        };
        let counts = rows
            .into_iter()
            .map(|row| {
                let value = row.0;
                debug_assert!(row.1 > 0);
                let count = row.1 as usize;
                StringCount { value, count }
            })
            .collect();
        Ok(FieldStrings::new(field, counts))
    }

    fn collection_stats(&self, collection_uid: &EntityUid) -> TracksResult<CollectionTrackStats> {
        let total_count = aux_track_collection::table
            .select(diesel::dsl::count_star())
            .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()))
            .first::<i64>(self.connection)? as usize;

        Ok(CollectionTrackStats { total_count })
    }
}

impl<'a> TrackAlbums for TrackRepository<'a> {
    fn count_tracks_by_album(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByAlbumParams,
        pagination: Pagination,
    ) -> TrackAlbumsResult<Vec<AlbumTracksCount>> {
        let mut target = aux_track_brief::table
            .select((
                aux_track_brief::album_title,
                aux_track_brief::album_artist,
                aux_track_brief::release_year,
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS count"),
            ))
            .group_by((
                aux_track_brief::album_title,
                aux_track_brief::album_artist,
                aux_track_brief::release_year,
            ))
            .into_boxed();

        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_brief::track_id.eq_any(track_id_subselect));
        }

        target = target.filter(aux_track_brief::album_title.is_not_null());
        target = target.filter(aux_track_brief::album_artist.is_not_null());
        if let Some(min_release_year) = params.min_release_year {
            target = target.filter(aux_track_brief::release_year.ge(min_release_year));
        }
        if let Some(max_release_year) = params.max_release_year {
            target = target.filter(aux_track_brief::release_year.le(max_release_year));
        }

        for sort_order in &params.ordering {
            let direction = sort_order.direction();
            use TrackSortField::*;
            use SortDirection::*;
            match sort_order.field() {
                AlbumTitle => match direction {
                    Ascending => {
                        target = target.then_order_by(aux_track_brief::album_title.asc());
                    }
                    Descending => {
                        target = target.then_order_by(aux_track_brief::album_title.desc())
                    }
                },
                AlbumArtist => match direction {
                    Ascending => {
                        target = target.then_order_by(aux_track_brief::track_artist.asc());
                    }
                    Descending => {
                        target = target.then_order_by(aux_track_brief::album_artist.desc());
                    }
                },
                ReleaseYear => match direction {
                    Ascending => {
                        target = target.then_order_by(aux_track_brief::release_year.asc());
                    }
                    Descending => {
                        target = target.then_order_by(aux_track_brief::release_year.desc());
                    }
                },
                field => log::warn!(
                    "Ignoring sort order by field {:?} when counting tracks by albums",
                    field
                ),
            }
        }
        target = target.then_order_by(sql::<diesel::sql_types::BigInt>("count").desc());

        // Pagination
        target = apply_pagination(target, pagination);

        let res =
            target.load::<(Option<String>, Option<String>, Option<i16>, i64)>(self.connection)?;

        Ok(res
            .into_iter()
            .map(|row| AlbumTracksCount {
                title: row.0,
                artist: row.1,
                release_year: row.2,
                count: row.3 as usize,
            })
            .collect())
    }
}

impl<'a> TrackTags for TrackRepository<'a> {
    fn count_tracks_by_tag_facet(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByTagFacetParams,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<TagFacetCount>> {
        let mut target = aux_track_tag::table
            .inner_join(aux_tag_facet::table)
            .select((
                aux_tag_facet::facet,
                sql::<diesel::sql_types::BigInt>("count(*) AS count"),
            ))
            .group_by(aux_track_tag::facet_id)
            .into_boxed();

        // Facet filtering
        if let Some(ref facets) = params.facets {
            target = target.filter(
                aux_tag_facet::facet.eq_any(facets.iter().map(AsRef::as_ref).map(String::as_str)),
            );
        }

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Ordering
        if params.ordering.is_empty() {
            target = target.then_order_by(sql::<diesel::sql_types::BigInt>("count").desc());
        } else {
            for sort_order in &params.ordering {
                let direction = sort_order.direction();
                use TagSortField::*;
                use SortDirection::*;
                match sort_order.field() {
                    Facet => {
                        let col = aux_tag_facet::facet;
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.asc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Count => {
                        let col = sql::<diesel::sql_types::BigInt>("count");
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.desc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    field => log::warn!(
                        "Ignoring sort order by field {:?} when counting tracks by tag facet",
                        field
                    ),
                }
            }
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(String, i64)>(self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| TagFacetCount::new(row.0.into(), row.1 as usize))
            .collect())
    }

    fn count_tracks_by_tag(
        &self,
        collection_uid: Option<&EntityUid>,
        params: &CountTracksByTagParams,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<TagCount>> {
        let mut target = aux_track_tag::table
            .left_outer_join(aux_tag_facet::table)
            .left_outer_join(aux_tag_label::table)
            .select((
                aux_tag_facet::facet.nullable(),
                aux_tag_label::label.nullable(),
                sql::<diesel::sql_types::Double>("AVG(score) AS avg_score"),
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS count"),
            ))
            .group_by((aux_track_tag::facet_id, aux_track_tag::label_id))
            .into_boxed();

        // Facet filtering
        if let Some(ref facets) = params.facets {
            let facets = facets.iter().map(AsRef::as_ref).map(String::as_str);
            target = if params.include_non_faceted_tags() {
                target.filter(
                    aux_tag_facet::facet
                        .eq_any(facets)
                        .or(aux_tag_facet::facet.is_null()),
                )
            } else {
                target.filter(aux_tag_facet::facet.eq_any(facets))
            };
        } else {
            // Include all faceted tags
            if !params.include_non_faceted_tags() {
                target = target.filter(aux_track_tag::facet_id.is_not_null());
            }
        }

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Ordering
        if params.ordering.is_empty() {
            target = target.then_order_by(sql::<diesel::sql_types::BigInt>("count").desc());
        } else {
            for sort_order in &params.ordering {
                let direction = sort_order.direction();
                use TagSortField::*;
                use SortDirection::*;
                match sort_order.field() {
                    Facet => {
                        let col = aux_tag_facet::facet;
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.asc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Label => {
                        let col = aux_tag_label::label;
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.asc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Score => {
                        let col = sql::<diesel::sql_types::Double>("avg_score");
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.desc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                    Count => {
                        let col = sql::<diesel::sql_types::BigInt>("count");
                        match direction {
                            Ascending => {
                                target = target.then_order_by(col.desc());
                            }
                            Descending => {
                                target = target.then_order_by(col.desc());
                            }
                        }
                    }
                }
            }
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(Option<String>, Option<String>, f64, i64)>(self.connection)?;
        Ok(rows
            .into_iter()
            .map(|row| TagCount {
                facet: row.0.map(Into::into),
                label: row.1.map(Into::into),
                avg_score: row.2.into(),
                count: row.3 as usize,
            })
            .collect())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
