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

mod schema;

pub mod util;

use self::models::*;

use self::schema::*;

use self::util::TrackRepositoryHelper;

use storage::util::*;

use chrono::NaiveDate;

use diesel;
use diesel::dsl::*;
use diesel::prelude::*;

use api::{
    album::*, collection::CollectionTrackStats,
    serde::{serialize_with_format, SerializationFormat, SerializedEntity}, track::*, *,
};

use aoide_core::{
    audio::*, domain::{entity::*, metadata::*, track::*},
};

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
    tag_filter: TagFilter,
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
    } else if let Some(facet) = tag_filter.facet {
        let subselect = aux_track_tag_facet::table
            .select(aux_track_tag_facet::id)
            .filter(aux_track_tag_facet::facet.eq(facet));
        select = select.filter(aux_track_tag::facet_id.eq_any(subselect));
    }

    // Filter tag term
    if let Some(term_condition) = tag_filter.term_condition {
        let (either_eq_or_like, modifier) = match term_condition.comparator {
            // Equal comparison
            StringComparator::Matches => (
                EitherEqualOrLike::Equal(term_condition.value),
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
    numeric_filter: NumericFilter,
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
        let prev_revision = entity.header().revision().clone();
        let next_revision = prev_revision.next();
        {
            let uid = entity.header().uid().clone();
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
                if rows_affected <= 0 {
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
        for replacement in replace_params.replacements.into_iter() {
            let uri_filter = UriFilter {
                condition: StringCondition {
                    comparator: StringComparator::Matches,
                    value: replacement.uri.clone(),
                    modifier: None,
                },
                modifier: None,
            };
            let locate_params = LocateTracksParams { uri_filter };
            let located_entities =
                self.locate_entities(collection_uid, &Pagination::default(), locate_params)?;
            // Ambiguous?
            if located_entities.len() > 1 {
                assert!(collection_uid.is_none());
                warn!(
                    "Found multiple tracks with URI '{}' in different collections",
                    replacement.uri
                );
                results.rejected.push(replacement.uri);
                continue;
            }
            if !replacement.track.is_valid() {
                warn!(
                    "Replacing track although it is not valid: {:?}",
                    replacement.track
                );
                // ...ignore issues and continue
            }
            // Update?
            if let Some(serialized_entity) = located_entities.first() {
                let entity = serialized_entity.deserialize::<TrackEntity>()?;
                let uid = entity.header().uid().clone();
                if entity.body() == &replacement.track {
                    debug!(
                        "Track '{}' is unchanged and does not need to be updated",
                        uid
                    );
                    results.skipped.push(*entity.header());
                    continue;
                }
                let replaced_entity = entity.replace_body(replacement.track);
                match self.update_entity(replaced_entity, format)? {
                    (_, None) => {
                        let msg = format!(
                            "Failed to update track '{}' due to internal race condition",
                            uid
                        );
                        //warn!(msg);
                        results.rejected.push(msg);
                    }
                    (_, Some(next_revision)) => {
                        let header = EntityHeader::new(uid, next_revision);
                        results.updated.push(header);
                    }
                };
                continue;
            }
            // Create?
            match replace_params.mode {
                ReplaceMode::UpdateOnly => {
                    info!(
                        "Track with URI '{}' does not exist and needs to be created",
                        replacement.uri
                    );
                    results.discarded.push(replacement.uri);
                    continue;
                }
                ReplaceMode::UpdateOrCreate => {
                    if let Some(collection_uid) = collection_uid {
                        // Check consistency to avoid unique constraint violations
                        // when inserting into the database.
                        match replacement.track.resource(collection_uid) {
                            Some(resource) => {
                                if resource.source.uri != replacement.uri {
                                    warn!(
                                        "Mismatching track URI: expected = '{}', actual = '{}'",
                                        replacement.uri, resource.source.uri
                                    );
                                    results.rejected.push(replacement.uri);
                                    continue;
                                }
                            }
                            None => {
                                warn!(
                                    "Track with URI '{}' does not belong to collection '{}'",
                                    replacement.uri, collection_uid
                                );
                                results.rejected.push(replacement.uri);
                                continue;
                            }
                        }
                    }
                    let entity = self.create_entity(replacement.track, format)?;
                    results.created.push(*entity.header())
                }
            };
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
        if rows_affected <= 0 {
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
            .map_err(|e| e.into())
    }

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
        locate_params: LocateTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>> {
        // URI filter
        let uri_condition = locate_params.uri_filter.condition;
        let (either_eq_or_like, modifier) = match uri_condition.comparator {
            // Equal comparison
            StringComparator::Matches => (
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
        };

        // A subselect has proven to be much more efficient than
        // joining the aux_track_resource table!!
        let mut track_id_subselect = aux_track_resource::table
            .select(aux_track_resource::track_id)
            .into_boxed();
        if let Some(collection_uid) = collection_uid {
            track_id_subselect = track_id_subselect
                .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()));
        };
        track_id_subselect = match either_eq_or_like {
            EitherEqualOrLike::Equal(eq) => match modifier {
                None => track_id_subselect.filter(aux_track_resource::source_uri.eq(eq)),
                Some(ConditionModifier::Not) => {
                    track_id_subselect.filter(aux_track_resource::source_uri.ne(eq))
                }
            },
            EitherEqualOrLike::Like(like) => match modifier {
                None => track_id_subselect
                    .filter(aux_track_resource::source_uri.like(like).escape('\\')),
                Some(ConditionModifier::Not) => track_id_subselect
                    .filter(aux_track_resource::source_uri.not_like(like).escape('\\')),
            },
        };

        let mut target = tbl_track::table
            .select(tbl_track::all_columns)
            .order_by(tbl_track::id) // preserve relative order of results
            .into_boxed();

        target = match locate_params.uri_filter.modifier {
            None => target.or_filter(tbl_track::id.eq_any(track_id_subselect)),
            Some(FilterModifier::Complement) => {
                target.or_filter(tbl_track::id.ne_all(track_id_subselect))
            }
        };

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableSerializedEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(|e| e.into())
    }

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: &Pagination,
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
            .inner_join(aux_track_resource::table)
            .inner_join(aux_track_overview::table)
            .inner_join(aux_track_summary::table)
            .into_boxed();

        if let Some(phrase_filter) = search_params.phrase_filter {
            // Escape wildcard character with backslash (see below)
            let escaped = phrase_filter
                .phrase
                .replace('\\', "\\\\")
                .replace('%', "\\%");
            let escaped_and_tokenized =
                escaped.split_whitespace().filter(|token| !token.is_empty());
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
                            aux_track_resource::source_uri_decoded
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Complement) => target.or_filter(
                            aux_track_resource::source_uri_decoded
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
                            aux_track_resource::media_type
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Complement) => target.or_filter(
                            aux_track_resource::media_type
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
                            aux_track_overview::track_title
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Complement) => target.or_filter(
                            aux_track_overview::track_title
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
                            aux_track_overview::album_title
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Complement) => target.or_filter(
                            aux_track_overview::album_title
                                .not_like(like_expr.clone())
                                .escape('\\'),
                        ),
                    };
                }

                // aux_track_summary (join)
                if phrase_filter.fields.is_empty()
                    || phrase_filter
                        .fields
                        .iter()
                        .any(|target| *target == PhraseField::TrackArtist)
                {
                    target = match phrase_filter.modifier {
                        None => target.or_filter(
                            aux_track_summary::track_artist
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Complement) => target.or_filter(
                            aux_track_summary::track_artist
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
                            aux_track_summary::album_artist
                                .like(like_expr.clone())
                                .escape('\\'),
                        ),
                        Some(FilterModifier::Complement) => target.or_filter(
                            aux_track_summary::album_artist
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
                    let subselect = aux_track_comment::table
                        .select(aux_track_comment::track_id)
                        .filter(aux_track_comment::text.like(like_expr.clone()).escape('\\'));
                    target = match phrase_filter.modifier {
                        None => target.or_filter(tbl_track::id.eq_any(subselect)),
                        Some(FilterModifier::Complement) => {
                            target.or_filter(tbl_track::id.ne_all(subselect))
                        }
                    };
                }
            }
        }

        for tag_filter in search_params.tag_filters.into_iter() {
            let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(tag_filter);
            target = match filter_modifier {
                None => target.filter(tbl_track::id.eq_any(subselect)),
                Some(FilterModifier::Complement) => target.filter(tbl_track::id.ne_all(subselect)),
            }
        }

        for numeric_filter in search_params.numeric_filters {
            target = match select_track_ids_from_profile_matching_numeric_filter(numeric_filter) {
                Some((subselect, filter_modifier)) => match filter_modifier {
                    None => target.filter(tbl_track::id.eq_any(subselect)),
                    Some(FilterModifier::Complement) => {
                        target.filter(tbl_track::id.ne_all(subselect))
                    }
                },
                None => match numeric_filter.field {
                    NumericField::DurationMs => match numeric_filter.condition.comparator {
                        NumericComparator::LessThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_duration_ms
                                        .lt(numeric_filter.condition.value),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_duration_ms
                                        .lt(numeric_filter.condition.value))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_duration_ms
                                        .ge(numeric_filter.condition.value),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_duration_ms
                                        .ge(numeric_filter.condition.value))),
                            },
                        },
                        NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_duration_ms
                                        .gt(numeric_filter.condition.value),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_duration_ms
                                        .gt(numeric_filter.condition.value))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_duration_ms
                                        .le(numeric_filter.condition.value),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_duration_ms
                                        .le(numeric_filter.condition.value))),
                            },
                        },
                        NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_duration_ms
                                        .eq(numeric_filter.condition.value),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_duration_ms
                                        .eq(numeric_filter.condition.value))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_duration_ms
                                        .ne(numeric_filter.condition.value),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_duration_ms
                                        .ne(numeric_filter.condition.value))),
                            },
                        },
                    },
                    NumericField::SampleRateHz => match numeric_filter.condition.comparator {
                        NumericComparator::LessThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_samplerate_hz
                                        .lt(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_samplerate_hz
                                        .lt(numeric_filter.condition.value as i32))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_samplerate_hz
                                        .ge(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_samplerate_hz
                                        .ge(numeric_filter.condition.value as i32))),
                            },
                        },
                        NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_samplerate_hz
                                        .gt(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_samplerate_hz
                                        .gt(numeric_filter.condition.value as i32))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_samplerate_hz
                                        .le(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_samplerate_hz
                                        .le(numeric_filter.condition.value as i32))),
                            },
                        },
                        NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_samplerate_hz
                                        .eq(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_samplerate_hz
                                        .eq(numeric_filter.condition.value as i32))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_samplerate_hz
                                        .ne(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_samplerate_hz
                                        .ne(numeric_filter.condition.value as i32))),
                            },
                        },
                    },
                    NumericField::BitRateBps => match numeric_filter.condition.comparator {
                        NumericComparator::LessThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_bitrate_bps
                                        .lt(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_bitrate_bps
                                        .lt(numeric_filter.condition.value as i32))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_bitrate_bps
                                        .ge(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_bitrate_bps
                                        .ge(numeric_filter.condition.value as i32))),
                            },
                        },
                        NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_bitrate_bps
                                        .gt(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_bitrate_bps
                                        .gt(numeric_filter.condition.value as i32))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_bitrate_bps
                                        .le(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_bitrate_bps
                                        .le(numeric_filter.condition.value as i32))),
                            },
                        },
                        NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_bitrate_bps
                                        .eq(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_bitrate_bps
                                        .eq(numeric_filter.condition.value as i32))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_bitrate_bps
                                        .ne(numeric_filter.condition.value as i32),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_bitrate_bps
                                        .ne(numeric_filter.condition.value as i32))),
                            },
                        },
                    },
                    NumericField::ChannelsCount => match numeric_filter.condition.comparator {
                        NumericComparator::LessThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_channels_count
                                        .lt(numeric_filter.condition.value as i16),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_channels_count
                                        .lt(numeric_filter.condition.value as i16))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_channels_count
                                        .ge(numeric_filter.condition.value as i16),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_channels_count
                                        .ge(numeric_filter.condition.value as i16))),
                            },
                        },
                        NumericComparator::GreaterThan => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_channels_count
                                        .gt(numeric_filter.condition.value as i16),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_channels_count
                                        .gt(numeric_filter.condition.value as i16))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_channels_count
                                        .le(numeric_filter.condition.value as i16),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_channels_count
                                        .le(numeric_filter.condition.value as i16))),
                            },
                        },
                        NumericComparator::EqualTo => match numeric_filter.condition.modifier {
                            None => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_channels_count
                                        .eq(numeric_filter.condition.value as i16),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_channels_count
                                        .eq(numeric_filter.condition.value as i16))),
                            },
                            Some(ConditionModifier::Not) => match numeric_filter.modifier {
                                None => target.filter(
                                    aux_track_resource::audio_channels_count
                                        .ne(numeric_filter.condition.value as i16),
                                ),
                                Some(FilterModifier::Complement) => target
                                    .filter(not(aux_track_resource::audio_channels_count
                                        .ne(numeric_filter.condition.value as i16))),
                            },
                        },
                    },
                    numeric_field => {
                        unreachable!("unhandled numeric filter field: {:?}", numeric_field)
                    }
                },
            };
        }

        // Collection filter
        if let Some(uid) = collection_uid {
            target = target.filter(aux_track_resource::collection_uid.eq(uid.as_ref()));
        };

        for sort_order in search_params.ordering {
            let direction = sort_order
                .direction
                .unwrap_or_else(|| TrackSort::default_direction(sort_order.field));
            target = match sort_order.field {
                field @ TrackSortField::InCollectionSince => {
                    if collection_uid.is_some() {
                        match direction {
                            SortDirection::Ascending => {
                                target.then_order_by(aux_track_resource::collection_since.asc())
                            }
                            SortDirection::Descending => {
                                target.then_order_by(aux_track_resource::collection_since.desc())
                            }
                        }
                    } else {
                        warn!("Cannot order by {:?} over multiple collections", field);
                        target
                    }
                }
                TrackSortField::LastRevisionedAt => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(tbl_track::rev_timestamp.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(tbl_track::rev_timestamp.desc())
                    }
                },
                TrackSortField::TrackTitle => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(aux_track_overview::track_title.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(aux_track_overview::track_title.desc())
                    }
                },
                TrackSortField::AlbumTitle => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(aux_track_overview::album_title.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(aux_track_overview::album_title.desc())
                    }
                },
                TrackSortField::ReleasedAt => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(aux_track_overview::released_at.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(aux_track_overview::released_at.desc())
                    }
                },
                TrackSortField::ReleasedBy => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(aux_track_overview::released_by.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(aux_track_overview::released_by.desc())
                    }
                },
                TrackSortField::TrackArtist => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(aux_track_summary::track_artist.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(aux_track_summary::track_artist.desc())
                    }
                },
                TrackSortField::AlbumArtist => match direction {
                    SortDirection::Ascending => {
                        target.then_order_by(aux_track_summary::album_artist.asc())
                    }
                    SortDirection::Descending => {
                        target.then_order_by(aux_track_summary::album_artist.desc())
                    }
                },
            }
        }
        // Finally order by PK to preserve the relative order of results
        // even if no sorting was requested.
        target = target.then_order_by(tbl_track::id);

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableSerializedEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(|e| e.into())
    }

    fn list_fields(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: &Pagination,
    ) -> TracksResult<StringFieldCounts> {
        let track_id_subselect = collection_uid.map(|collection_uid| {
            aux_track_resource::table
                .select(aux_track_resource::track_id)
                .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()))
        });
        let rows = match field {
            StringField::MediaType => {
                let mut target = aux_track_resource::table
                    .select((
                        aux_track_resource::media_type,
                        sql::<diesel::sql_types::BigInt>("count(*) AS count"),
                    ))
                    .group_by(aux_track_resource::media_type)
                    .order_by(sql::<diesel::sql_types::BigInt>("count").desc())
                    .then_order_by(aux_track_resource::media_type)
                    .into_boxed();
                if let Some(collection_uid) = collection_uid {
                    target = target
                        .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()));
                }

                // Pagination
                target = apply_pagination(target, pagination);

                let rows = target.load::<(String, i64)>(self.connection)?;
                // TODO: Remove this transformation and select media_type
                // as a nullable column?!
                rows.into_iter()
                    .map(|(media_type, count)| (Some(media_type), count))
                    .collect()
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

    fn collection_stats(&self, collection_uid: &EntityUid) -> TracksResult<CollectionTrackStats> {
        let total_count = aux_track_resource::table
            .select(diesel::dsl::count_star())
            .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()))
            .first::<i64>(self.connection)? as usize;

        let sum_duration_ms = aux_track_resource::table
            .select(diesel::dsl::sum(aux_track_resource::audio_duration_ms))
            .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()))
            .first::<Option<f64>>(self.connection)?;
        let total_duration = sum_duration_ms
            .map(|ms| DurationMs::from_ms(ms))
            .unwrap_or(DurationMs::EMPTY);

        Ok(CollectionTrackStats {
            total_count,
            total_duration,
        })
    }
}

impl<'a> TrackTags for TrackRepository<'a> {
    fn list_tag_facets(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: &Pagination,
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
            let track_id_subselect = aux_track_resource::table
                .select(aux_track_resource::track_id)
                .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
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

    fn list_tags(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: &Pagination,
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
            let track_id_subselect = aux_track_resource::table
                .select(aux_track_resource::track_id)
                .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()));
            target = target.filter(aux_track_tag::track_id.eq_any(track_id_subselect));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        let rows = target.load::<(f64, String, Option<String>, i64)>(self.connection)?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows.into_iter() {
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
        pagination: &Pagination,
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
            let track_id_subselect = aux_track_resource::table
                .select(aux_track_resource::track_id)
                .filter(aux_track_resource::collection_uid.eq(collection_uid.as_ref()));
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
        for row in rows.into_iter() {
            let artist = row.0;
            let title = row.1;
            let min_released_at = row.2;
            let max_released_at = row.3;
            let released_between = match (min_released_at, max_released_at) {
                (Some(min), Some(max)) => Some(NaiveDateRange { min, max }),
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
                released_between,
                total_tracks,
            };
            result.push(album_summary);
        }
        Ok(result)
    }
}
