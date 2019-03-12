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

use self::{
    models::*, schema::*, track_search_query_transform::TrackSearchQueryTransform,
    util::TrackRepositoryHelper,
};

use crate::{
    api::{
        album::*,
        collection::CollectionTrackStats,
        serde::{serialize_with_format, SerializationFormat, SerializedEntity},
        track::*,
        *,
    },
    storage::util::*,
};

use crate::core::{entity::*, metadata::*, prelude::*, track::*};

use chrono::NaiveDate;

use diesel::dsl::*;

///////////////////////////////////////////////////////////////////////
/// Modules
///////////////////////////////////////////////////////////////////////
mod models;

mod schema;

mod track_search_query_transform;

pub mod util;

#[cfg(test)]
mod tests;

///////////////////////////////////////////////////////////////////////
/// TrackRepository
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

    // Filter tag facet
    if tag_filter.facet == TagFilter::no_facet() {
        select = select.filter(aux_track_tag::facet_id.is_null());
    } else if let Some(ref facet) = tag_filter.facet {
        let subselect = aux_track_tag_facet::table
            .select(aux_track_tag_facet::id)
            .filter(aux_track_tag_facet::facet.eq(facet));
        select = select.filter(aux_track_tag::facet_id.eq_any(subselect));
    }

    // Filter tag term
    if let Some(ref term_condition) = tag_filter.term_condition {
        let (either_eq_or_like, modifier) = match term_condition.comparator {
            // Equal comparison
            StringComparator::Equals => (
                EitherEqualOrLike::Equal(term_condition.value.clone()),
                term_condition.modifier,
            ),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringComparator::StartsWith => (
                EitherEqualOrLike::Like(format!(
                    "{}%",
                    term_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                term_condition.modifier,
            ),
            StringComparator::EndsWith => (
                EitherEqualOrLike::Like(format!(
                    "%{}",
                    term_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                term_condition.modifier,
            ),
            StringComparator::Contains => (
                EitherEqualOrLike::Like(format!(
                    "%{}%",
                    term_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                term_condition.modifier,
            ),
            StringComparator::Matches => (
                EitherEqualOrLike::Like(
                    term_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%"),
                ),
                term_condition.modifier,
            ),
        };
        select = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => {
                    let subselect = aux_track_tag_term::table
                        .select(aux_track_tag_term::id)
                        .filter(aux_track_tag_term::term.eq(eq));
                    select.filter(aux_track_tag::term_id.eq_any(subselect))
                }
                Some(ConditionModifier::Not) => {
                    let subselect = aux_track_tag_term::table
                        .select(aux_track_tag_term::id)
                        .filter(aux_track_tag_term::term.ne(eq));
                    select.filter(aux_track_tag::term_id.eq_any(subselect))
                }
            },
            EitherEqualOrLike::Like(like) => match modifier {
                None => {
                    let subselect = aux_track_tag_term::table
                        .select(aux_track_tag_term::id)
                        .filter(aux_track_tag_term::term.like(like).escape('\\'));
                    select.filter(aux_track_tag::term_id.eq_any(subselect))
                }
                Some(ConditionModifier::Not) => {
                    let subselect = aux_track_tag_term::table
                        .select(aux_track_tag_term::id)
                        .filter(aux_track_tag_term::term.not_like(like).escape('\\'));
                    select.filter(aux_track_tag::term_id.eq_any(subselect))
                }
            },
        };
    }

    // Filter tag score
    if let Some(score_condition) = tag_filter.score_condition {
        select = match score_condition.comparator {
            NumericComparator::LessThan => match score_condition.modifier {
                None => select.filter(aux_track_tag::score.lt(score_condition.value)),
                Some(ConditionModifier::Not) => {
                    select.filter(aux_track_tag::score.ge(score_condition.value))
                }
            },
            NumericComparator::GreaterThan => match score_condition.modifier {
                None => select.filter(aux_track_tag::score.gt(score_condition.value)),
                Some(ConditionModifier::Not) => {
                    select.filter(aux_track_tag::score.le(score_condition.value))
                }
            },
            NumericComparator::EqualTo => match score_condition.modifier {
                None => select.filter(aux_track_tag::score.eq(score_condition.value)),
                Some(ConditionModifier::Not) => {
                    select.filter(aux_track_tag::score.ne(score_condition.value))
                }
            },
        };
    }

    (select, tag_filter.modifier)
}

fn select_track_ids_from_profile_matching_numeric_filter<'a, DB>(
    numeric_filter: &NumericFilter,
) -> Option<(
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        aux_track_profile::table,
        DB,
    >,
    Option<FilterModifier>,
)>
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = aux_track_profile::table
        .select(aux_track_profile::track_id)
        .into_boxed();

    select = match numeric_filter.field {
        NumericField::TempoBpm => {
            match numeric_filter.condition.comparator {
                NumericComparator::LessThan => match numeric_filter.condition.modifier {
                    None => select
                        .filter(aux_track_profile::tempo_bpm.lt(numeric_filter.condition.value)),
                    Some(ConditionModifier::Not) => select
                        .filter(aux_track_profile::tempo_bpm.ge(numeric_filter.condition.value)),
                },
                NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                    None => select
                        .filter(aux_track_profile::tempo_bpm.gt(numeric_filter.condition.value)),
                    Some(ConditionModifier::Not) => select
                        .filter(aux_track_profile::tempo_bpm.le(numeric_filter.condition.value)),
                },
                NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                    None => select
                        .filter(aux_track_profile::tempo_bpm.eq(numeric_filter.condition.value)),
                    Some(ConditionModifier::Not) => select
                        .filter(aux_track_profile::tempo_bpm.ne(numeric_filter.condition.value)),
                },
            }
        }
        NumericField::KeySigCode => match numeric_filter.condition.comparator {
            NumericComparator::LessThan => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::key_sig_code.lt(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::key_sig_code.ge(numeric_filter.condition.value as i16),
                ),
            },
            NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::key_sig_code.gt(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::key_sig_code.le(numeric_filter.condition.value as i16),
                ),
            },
            NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::key_sig_code.eq(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::key_sig_code.ne(numeric_filter.condition.value as i16),
                ),
            },
        },
        NumericField::TimeSigTop => match numeric_filter.condition.comparator {
            NumericComparator::LessThan => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::time_sig_top.lt(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::time_sig_top.ge(numeric_filter.condition.value as i16),
                ),
            },
            NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::time_sig_top.gt(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::time_sig_top.le(numeric_filter.condition.value as i16),
                ),
            },
            NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::time_sig_top.eq(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::time_sig_top.ne(numeric_filter.condition.value as i16),
                ),
            },
        },
        NumericField::TimeSigBottom => match numeric_filter.condition.comparator {
            NumericComparator::LessThan => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::time_sig_bottom.lt(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::time_sig_bottom.ge(numeric_filter.condition.value as i16),
                ),
            },
            NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::time_sig_bottom.gt(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::time_sig_bottom.le(numeric_filter.condition.value as i16),
                ),
            },
            NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                None => select.filter(
                    aux_track_profile::time_sig_bottom.eq(numeric_filter.condition.value as i16),
                ),
                Some(ConditionModifier::Not) => select.filter(
                    aux_track_profile::time_sig_bottom.ne(numeric_filter.condition.value as i16),
                ),
            },
        },
        _ => return None,
    };

    Some((select, numeric_filter.modifier))
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
                        .and(tbl_track::rev_ordinal.eq(prev_revision.ordinal() as i64))
                        .and(tbl_track::rev_timestamp.eq(prev_revision.timestamp().naive_utc())),
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
            let uri_filter = UriFilter {
                condition: StringCondition {
                    comparator: StringComparator::Equals,
                    value: replacement.uri.clone(),
                    modifier: None,
                },
                modifier: None,
            };
            let locate_params = LocateTracksParams { uri_filter };
            // Workaround for performance regression:
            // * Locate entities for any collection
            // * Post-filtering of located entities by collection (see below)
            // See also: https://gitlab.com/uklotzde/aoide-rs/issues/12
            let located_entities = self.locate_entities(
                /*collection_uid*/ None,
                Pagination::default(),
                locate_params,
            )?;
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
            // Workaround for performance regression:
            // * Post-filtering of located entities by collection (see above)
            // See also: https://gitlab.com/uklotzde/aoide-rs/issues/12
            let deserialized_entities: Vec<TrackEntity> = deserialized_entities
                .into_iter()
                .filter(|entity| match collection_uid {
                    Some(collection_uid) => TrackCollection::filter_slice_by_uid(
                        &entity.body().collections,
                        collection_uid,
                    )
                    .is_some(),
                    None => true,
                })
                .collect();
            // Ambiguous?
            if deserialized_entities.len() > 1 {
                log::warn!("Found multiple tracks with URI '{}'", replacement.uri);
                results.rejected.push(replacement.uri);
                continue;
            }
            if !replacement.track.is_valid() {
                log::warn!(
                    "Accepting replacement track even though it is not valid: {:?}",
                    replacement.track
                );
                // ...ignore semantic issues and continue
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
            .map(|o| o.map(|o| o.into()))
            .map_err(Into::into)
    }

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        locate_params: LocateTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>> {
        // URI filter
        let uri_condition = locate_params.uri_filter.condition;
        let (either_eq_or_like, modifier) = match uri_condition.comparator {
            // Equal comparison
            StringComparator::Equals => (
                EitherEqualOrLike::Equal(uri_condition.value),
                uri_condition.modifier,
            ),
            // Like comparison: Escape wildcard character with backslash (see below)
            StringComparator::StartsWith => (
                EitherEqualOrLike::Like(format!(
                    "{}%",
                    uri_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                uri_condition.modifier,
            ),
            StringComparator::EndsWith => (
                EitherEqualOrLike::Like(format!(
                    "%{}",
                    uri_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                uri_condition.modifier,
            ),
            StringComparator::Contains => (
                EitherEqualOrLike::Like(format!(
                    "%{}%",
                    uri_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%")
                )),
                uri_condition.modifier,
            ),
            StringComparator::Matches => (
                EitherEqualOrLike::Like(
                    uri_condition
                        .value
                        .replace('\\', "\\\\")
                        .replace('%', "\\%"),
                ),
                uri_condition.modifier,
            ),
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
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => track_id_subselect.filter(aux_track_source::uri.eq(eq)),
                Some(ConditionModifier::Not) => {
                    track_id_subselect.filter(aux_track_source::uri.ne(eq))
                }
            },
            EitherEqualOrLike::Like(like) => match modifier {
                None => track_id_subselect.filter(aux_track_source::uri.like(like).escape('\\')),
                Some(ConditionModifier::Not) => {
                    track_id_subselect.filter(aux_track_source::uri.not_like(like).escape('\\'))
                }
            },
        };
        target = match locate_params.uri_filter.modifier {
            None => target.filter(tbl_track::id.eq_any(track_id_subselect)),
            Some(FilterModifier::Complement) => {
                target.filter(tbl_track::id.ne_all(track_id_subselect))
            }
        };

        // Collection filtering
        // TODO: The second subselect that has been introduced when splitting
        // aux_track_resource into aux_track_collection and aux_track_source
        // slows down the query substantially although all columns are properly
        // indexed! How could this be to optimized?
        // See also: https://gitlab.com/uklotzde/aoide-rs/issues/12
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
            .map(|v| v.into_iter().map(|r| r.into()).collect())
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
            .inner_join(aux_track_overview::table)
            .inner_join(aux_track_summary::table)
            .left_outer_join(aux_track_source::table)
            .left_outer_join(aux_track_collection::table)
            .into_boxed();

        if let Some(ref phrase_filter) = search_params.phrase_filter {
            target = phrase_filter.apply_to_query(target, collection_uid);
        }

        for tag_filter in &search_params.tag_filters {
            target = tag_filter.apply_to_query(target, collection_uid);
        }

        for numeric_filter in &search_params.numeric_filters {
            target = numeric_filter.apply_to_query(target, collection_uid);
        }

        if let Some(ref filter) = search_params.filter {
            target = filter.apply_to_query(target, collection_uid);
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

        target
            .load::<QueryableSerializedEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(Into::into)
    }

    fn list_fields(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: Pagination,
    ) -> TracksResult<StringFieldCounts> {
        let track_id_subselect = collection_uid.map(|collection_uid| {
            aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()))
        });
        let rows = match field {
            StringField::AlbumArtist => {
                let mut target = aux_track_summary::table
                    .select((
                        aux_track_summary::album_artist,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_summary::album_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_summary::album_artist)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_summary::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::AlbumTitle => {
                let mut target = aux_track_overview::table
                    .select((
                        aux_track_overview::album_title,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_overview::album_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_overview::album_title)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_overview::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::SourceUri => {
                let mut target = aux_track_source::table
                    .select((
                        aux_track_source::uri_decoded,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_source::uri_decoded)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_source::uri_decoded)
                    .into_boxed();

                // Collection filtering
                if let Some(collection_uid) = collection_uid {
                    let track_id_subselect = aux_track_collection::table
                        .select(aux_track_collection::track_id)
                        .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
                    target = target.filter(aux_track_source::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                let rows = target.load::<(String, i64)>(self.connection)?;
                // TODO: Remove this transformation and select source_uri_decoded
                // as a nullable column?!
                rows.into_iter()
                    .map(|(source_uri_decoded, count)| (Some(source_uri_decoded), count))
                    .collect()
            }
            StringField::SourceType => {
                let mut target = aux_track_source::table
                    .select((
                        aux_track_source::content_type,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_source::content_type)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_source::content_type)
                    .into_boxed();

                // Collection filtering
                if let Some(collection_uid) = collection_uid {
                    let track_id_subselect = aux_track_collection::table
                        .select(aux_track_collection::track_id)
                        .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
                    target = target.filter(aux_track_source::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                let rows = target.load::<(String, i64)>(self.connection)?;
                // TODO: Remove this transformation and select content_type
                // as a nullable column?!
                rows.into_iter()
                    .map(|(content_type, count)| (Some(content_type), count))
                    .collect()
            }
            StringField::TrackArtist => {
                let mut target = aux_track_summary::table
                    .select((
                        aux_track_summary::track_artist,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_summary::track_artist)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_summary::track_artist)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_summary::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
            StringField::TrackTitle => {
                let mut target = aux_track_overview::table
                    .select((
                        aux_track_overview::track_title,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_overview::track_title)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_overview::track_title)
                    .into_boxed();
                if let Some(track_id_subselect) = track_id_subselect {
                    target = target.filter(aux_track_overview::track_id.eq_any(track_id_subselect));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                target.load::<(Option<String>, i64)>(self.connection)?
            }
        };
        let mut counts = Vec::with_capacity(rows.len());
        for row in rows {
            let value = row.0;
            debug_assert!(row.1 > 0);
            let count = row.1 as usize;
            counts.push(StringCount { value, count });
        }
        Ok(StringFieldCounts { field, counts })
    }

    fn collection_stats(&self, collection_uid: &EntityUid) -> TracksResult<CollectionTrackStats> {
        let total_count = aux_track_collection::table
            .select(diesel::dsl::count_star())
            .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()))
            .first::<i64>(self.connection)? as usize;

        Ok(CollectionTrackStats { total_count })
    }
}

impl<'a> TrackTags for TrackRepository<'a> {
    fn list_tag_facets(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<TagFacetCount>> {
        let mut target = aux_track_tag::table
            .left_outer_join(aux_track_tag_facet::table)
            .select((
                sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("facet"),
                sql::<diesel::sql_types::BigInt>("count(*) AS count"),
            ))
            .group_by(aux_track_tag::facet_id)
            .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Facet Filtering
        target = match facets {
            Some(facets) => {
                if facets.is_empty() {
                    target.filter(aux_track_tag::facet_id.is_null())
                } else {
                    let filtered = target.filter(aux_track_tag_facet::facet.eq_any(facets));
                    if facets.iter().any(|facet| facet.is_empty()) {
                        // Empty facets are interpreted as null, just like an empty vector
                        filtered.or_filter(aux_track_tag::facet_id.is_null())
                    } else {
                        filtered
                    }
                }
            }
            None => target,
        };

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(Option<String>, i64)>(self.connection)?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(TagFacetCount {
                facet: row.0,
                count: row.1 as usize,
            });
        }

        Ok(result)
    }

    fn list_tags(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<ScoredTagCount>> {
        let mut target = aux_track_tag::table
            .left_outer_join(aux_track_tag_term::table)
            .left_outer_join(aux_track_tag_facet::table)
            .select((
                sql::<diesel::sql_types::Double>("AVG(score) AS score"),
                aux_track_tag_term::term,
                // The joined 'facet' column becomes nullable
                sql::<diesel::sql_types::Nullable<diesel::sql_types::Text>>("facet"),
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS count"),
            ))
            .group_by((aux_track_tag::term_id, aux_track_tag::facet_id))
            .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
            .into_boxed();

        // Facet Filtering
        target = match facets {
            Some(facets) => {
                if facets.is_empty() {
                    target.filter(aux_track_tag::facet_id.is_null())
                } else {
                    let filtered = target.filter(aux_track_tag_facet::facet.eq_any(facets));
                    if facets.iter().any(|facet| facet.is_empty()) {
                        // Empty facets are interpreted as null, just like an empty vector
                        filtered.or_filter(aux_track_tag::facet_id.is_null())
                    } else {
                        filtered
                    }
                }
            }
            None => target,
        };

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(f64, String, Option<String>, i64)>(self.connection)?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(ScoredTagCount {
                tag: ScoredTag::new(row.0, row.1, row.2),
                count: row.3 as usize,
            });
        }

        Ok(result)
    }
}

impl<'a> Albums for TrackRepository<'a> {
    fn list_albums(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
    ) -> AlbumsResult<Vec<AlbumSummary>> {
        let mut target = aux_track_summary::table
            .inner_join(aux_track_overview::table)
            .select((
                aux_track_summary::album_artist,
                aux_track_overview::album_title,
                sql::<diesel::sql_types::Date>("MIN(released_at) AS min_released_at").nullable(),
                sql::<diesel::sql_types::Date>("MAX(released_at) AS max_released_at").nullable(),
                sql::<diesel::sql_types::BigInt>("COUNT(*) AS total_tracks"),
            ))
            .group_by((
                aux_track_summary::album_artist,
                aux_track_overview::album_title,
            ))
            .order_by((
                aux_track_summary::album_artist,
                sql::<diesel::sql_types::Date>("max_released_at").desc(),
                aux_track_overview::album_title,
            ))
            .into_boxed();

        // Collection filtering
        if let Some(collection_uid) = collection_uid {
            let track_id_subselect = aux_track_collection::table
                .select(aux_track_collection::track_id)
                .filter(aux_track_collection::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_summary::track_id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(
            Option<String>,
            Option<String>,
            Option<NaiveDate>,
            Option<NaiveDate>,
            i64,
        )>(self.connection)?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let artist = row.0;
            let title = row.1;
            let min_released_at = row.2;
            let max_released_at = row.3;
            let released_at = match (min_released_at, max_released_at) {
                (Some(earliest), Some(latest)) => Some(ReleasedAtDateRange { earliest, latest }),
                (_, _) => {
                    debug_assert!(min_released_at.is_none());
                    debug_assert!(max_released_at.is_none());
                    None
                }
            };
            debug_assert!(row.4 > 0);
            let total_tracks = row.4 as usize;
            let album_summary = AlbumSummary {
                artist,
                title,
                released_at,
                total_tracks,
            };
            result.push(album_summary);
        }
        Ok(result)
    }
}
