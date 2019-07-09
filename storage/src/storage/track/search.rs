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

///////////////////////////////////////////////////////////////////////

// TODO: How can we remove this ugly type alias definition?
type TrackSearchBoxedQuery<'a> = diesel::query_builder::BoxedSelectStatement<
    'a,
    (
        diesel::sql_types::BigInt,
        diesel::sql_types::Binary,
        diesel::sql_types::BigInt,
        diesel::sql_types::BigInt,
        diesel::sql_types::SmallInt,
        diesel::sql_types::Integer,
        diesel::sql_types::Integer,
        diesel::sql_types::Binary,
    ),
    diesel::query_source::joins::JoinOn<
        diesel::query_source::joins::Join<
            diesel::query_source::joins::JoinOn<
                diesel::query_source::joins::Join<
                    diesel::query_source::joins::JoinOn<
                        diesel::query_source::joins::Join<
                            tbl_track::table,
                            aux_track_brief::table,
                            diesel::query_source::joins::Inner,
                        >,
                        diesel::expression::operators::Eq<
                            diesel::expression::nullable::Nullable<
                                aux_track_brief::columns::track_id,
                            >,
                            diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                        >,
                    >,
                    aux_track_source::table,
                    diesel::query_source::joins::LeftOuter,
                >,
                diesel::expression::operators::Eq<
                    diesel::expression::nullable::Nullable<aux_track_source::columns::track_id>,
                    diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                >,
            >,
            aux_track_collection::table,
            diesel::query_source::joins::LeftOuter,
        >,
        diesel::expression::operators::Eq<
            diesel::expression::nullable::Nullable<aux_track_collection::columns::track_id>,
            diesel::expression::nullable::Nullable<tbl_track::columns::id>,
        >,
    >,
    diesel::sqlite::Sqlite,
>;

type TrackSearchQuery = diesel::query_source::joins::JoinOn<
    diesel::query_source::joins::Join<
        diesel::query_source::joins::JoinOn<
            diesel::query_source::joins::Join<
                diesel::query_source::joins::JoinOn<
                    diesel::query_source::joins::Join<
                        tbl_track::table,
                        aux_track_brief::table,
                        diesel::query_source::joins::Inner,
                    >,
                    diesel::expression::operators::Eq<
                        diesel::expression::nullable::Nullable<aux_track_brief::columns::track_id>,
                        diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                    >,
                >,
                aux_track_source::table,
                diesel::query_source::joins::LeftOuter,
            >,
            diesel::expression::operators::Eq<
                diesel::expression::nullable::Nullable<aux_track_source::columns::track_id>,
                diesel::expression::nullable::Nullable<tbl_track::columns::id>,
            >,
        >,
        aux_track_collection::table,
        diesel::query_source::joins::LeftOuter,
    >,
    diesel::expression::operators::Eq<
        diesel::expression::nullable::Nullable<aux_track_collection::columns::track_id>,
        diesel::expression::nullable::Nullable<tbl_track::columns::id>,
    >,
>;

type TrackSearchBoxedExpression<'a> = Box<
    BoxableExpression<TrackSearchQuery, diesel::sqlite::Sqlite, SqlType = diesel::sql_types::Bool>
        + 'a,
>;

// TODO: replace with "True"
fn dummy_true_expression() -> TrackSearchBoxedExpression<'static> {
    Box::new(tbl_track::id.is_not_null()) // always true
}

// TODO: replace with "False"
fn dummy_false_expression() -> TrackSearchBoxedExpression<'static> {
    Box::new(tbl_track::id.is_null()) // always false
}

pub trait TrackSearchBoxedExpressionBuilder {
    fn build_expression<'a>(
        &'a self,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a>;
}

pub trait TrackSearchQueryTransform {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a>;
}

impl TrackSearchQueryTransform for TrackSortOrder {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        let direction = self
            .direction
            .unwrap_or_else(|| TrackSortOrder::default_direction(self.field));
        match self.field {
            field @ TrackSortField::InCollectionSince => {
                if collection_uid.is_some() {
                    match direction {
                        SortDirection::Ascending => {
                            query.then_order_by(aux_track_collection::since.asc())
                        }
                        SortDirection::Descending => {
                            query.then_order_by(aux_track_collection::since.desc())
                        }
                    }
                } else {
                    log::warn!("Cannot order by {:?} over multiple collections", field);
                    query
                }
            }
            TrackSortField::LastRevisionedAt => match direction {
                SortDirection::Ascending => query.then_order_by(tbl_track::rev_ts.asc()),
                SortDirection::Descending => query.then_order_by(tbl_track::rev_ts.desc()),
            },
            TrackSortField::TrackTitle => match direction {
                SortDirection::Ascending => query.then_order_by(aux_track_brief::track_title.asc()),
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::track_title.desc())
                }
            },
            TrackSortField::TrackArtist => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_brief::track_artist.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::track_artist.desc())
                }
            },
            TrackSortField::TrackIndex => match direction {
                SortDirection::Ascending => query.then_order_by(aux_track_brief::track_index.asc()),
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::track_index.desc())
                }
            },
            TrackSortField::TrackCount => match direction {
                SortDirection::Ascending => query.then_order_by(aux_track_brief::track_count.asc()),
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::track_count.desc())
                }
            },
            TrackSortField::AlbumTitle => match direction {
                SortDirection::Ascending => query.then_order_by(aux_track_brief::album_title.asc()),
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::album_title.desc())
                }
            },
            TrackSortField::AlbumArtist => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_brief::track_artist.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::album_artist.desc())
                }
            },
            TrackSortField::ReleaseYear => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_brief::release_year.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::release_year.desc())
                }
            },
            TrackSortField::MusicTempo => match direction {
                SortDirection::Ascending => query.then_order_by(aux_track_brief::music_tempo.asc()),
                SortDirection::Descending => {
                    query.then_order_by(aux_track_brief::music_tempo.desc())
                }
            },
        }
    }
}

impl TrackSearchBoxedExpressionBuilder for PhraseFilter {
    fn build_expression<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        // Escape wildcard character with backslash (see below)
        let escaped_terms: Vec<_> = self
            .terms
            .iter()
            .map(|t| t.replace('\\', "\\\\").replace('%', "\\%"))
            .collect();
        let escaped_terms_str_len = escaped_terms.iter().fold(0, |len, term| len + term.len());
        // TODO: Use Rc<String> to avoid cloning strings?
        let like_expr = if escaped_terms_str_len > 0 {
            let mut like_expr = escaped_terms.iter().fold(
                String::with_capacity(escaped_terms_str_len + escaped_terms.len() + 1),
                |mut like_expr, term| {
                    // Prepend wildcard character before each part
                    like_expr.push('%');
                    like_expr.push_str(term);
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

        let mut or_expression = dummy_false_expression();
        // aux_track_source (join)
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::SourceUri)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_source::uri_decoded.is_null())
                        .or(aux_track_source::uri_decoded.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_source::uri_decoded
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::ContentType)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_source::uri_decoded.is_null())
                        .or(aux_track_source::uri_decoded.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_source::uri_decoded
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        // aux_track_brief (join)
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::TrackTitle)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_brief::track_title.is_null())
                        .or(aux_track_brief::track_title.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_brief::track_title
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::TrackArtist)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_brief::track_artist.is_null())
                        .or(aux_track_brief::track_artist.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_brief::track_artist
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::TrackComposer)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_brief::track_composer.is_null())
                        .or(aux_track_brief::track_composer.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_brief::track_composer
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::AlbumTitle)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_brief::album_title.is_null())
                        .or(aux_track_brief::album_title.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_brief::album_title
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        if self.fields.is_empty()
            || self
                .fields
                .iter()
                .any(|target| *target == StringField::AlbumArtist)
        {
            or_expression = if like_expr.is_empty() {
                Box::new(
                    or_expression
                        .or(aux_track_brief::album_artist.is_null())
                        .or(aux_track_brief::album_artist.eq(String::default())),
                )
            } else {
                Box::new(
                    or_expression.or(aux_track_brief::album_artist
                        .like(like_expr.clone())
                        .escape('\\')),
                )
            };
        }
        or_expression
    }
}

impl TrackSearchBoxedExpressionBuilder for NumericFilter {
    fn build_expression<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        match self.field {
            NumericField::AudioDuration => match self.value {
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_source::audio_duration.lt(value))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_source::audio_duration.le(value))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_source::audio_duration.gt(value))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_source::audio_duration.ge(value))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_duration.eq(value))
                    } else {
                        Box::new(aux_track_source::audio_duration.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_duration.ne(value))
                    } else {
                        Box::new(aux_track_source::audio_duration.is_not_null())
                    }
                }
            },
            NumericField::AudioSampleRate => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i32
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_source::audio_samplerate.lt(value as i32))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_source::audio_samplerate.le(value as i32))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_source::audio_samplerate.gt(value as i32))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_source::audio_samplerate.ge(value as i32))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_samplerate.eq(value as i32))
                    } else {
                        Box::new(aux_track_source::audio_samplerate.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_samplerate.ne(value as i32))
                    } else {
                        Box::new(aux_track_source::audio_samplerate.is_not_null())
                    }
                }
            },
            NumericField::AudioBitRate => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i32
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_source::audio_bitrate.lt(value as i32))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_source::audio_bitrate.le(value as i32))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_source::audio_bitrate.gt(value as i32))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_source::audio_bitrate.ge(value as i32))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_bitrate.eq(value as i32))
                    } else {
                        Box::new(aux_track_source::audio_bitrate.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_bitrate.ne(value as i32))
                    } else {
                        Box::new(aux_track_source::audio_bitrate.is_not_null())
                    }
                }
            },
            NumericField::AudioChannelCount => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i16
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_source::audio_channel_count.lt(value as i16))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_source::audio_channel_count.le(value as i16))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_source::audio_channel_count.gt(value as i16))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_source::audio_channel_count.ge(value as i16))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_channel_count.eq(value as i16))
                    } else {
                        Box::new(aux_track_source::audio_channel_count.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_channel_count.ne(value as i16))
                    } else {
                        Box::new(aux_track_source::audio_channel_count.is_not_null())
                    }
                }
            },
            NumericField::AudioLoudness => match self.value {
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_source::audio_loudness.lt(value))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_source::audio_loudness.le(value))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_source::audio_loudness.gt(value))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_source::audio_loudness.ge(value))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_loudness.eq(value))
                    } else {
                        Box::new(aux_track_source::audio_loudness.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_source::audio_loudness.ne(value))
                    } else {
                        Box::new(aux_track_source::audio_loudness.is_not_null())
                    }
                }
            },
            NumericField::TrackIndex => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i16
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_brief::track_index.lt(value as i16))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_brief::track_index.le(value as i16))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_brief::track_index.gt(value as i16))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_brief::track_index.ge(value as i16))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::track_index.eq(value as i16))
                    } else {
                        Box::new(aux_track_brief::track_index.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::track_index.ne(value as i16))
                    } else {
                        Box::new(aux_track_brief::track_index.is_not_null())
                    }
                }
            },
            NumericField::TrackCount => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i16
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_brief::track_count.lt(value as i16))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_brief::track_count.le(value as i16))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_brief::track_count.gt(value as i16))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_brief::track_count.ge(value as i16))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::track_count.eq(value as i16))
                    } else {
                        Box::new(aux_track_brief::track_count.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::track_count.ne(value as i16))
                    } else {
                        Box::new(aux_track_brief::track_count.is_not_null())
                    }
                }
            },
            NumericField::ReleaseYear => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i32
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_brief::release_year.lt(value as i16))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_brief::release_year.le(value as i16))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_brief::release_year.gt(value as i16))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_brief::release_year.ge(value as i16))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::release_year.eq(value as i16))
                    } else {
                        Box::new(aux_track_brief::release_year.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::release_year.ne(value as i16))
                    } else {
                        Box::new(aux_track_brief::release_year.is_not_null())
                    }
                }
            },
            NumericField::MusicTempo => match self.value {
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_brief::music_tempo.lt(value))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_brief::music_tempo.le(value))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_brief::music_tempo.gt(value))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_brief::music_tempo.ge(value))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::music_tempo.eq(value))
                    } else {
                        Box::new(aux_track_brief::music_tempo.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::music_tempo.ne(value))
                    } else {
                        Box::new(aux_track_brief::music_tempo.is_not_null())
                    }
                }
            },
            NumericField::MusicKey => match self.value {
                // TODO: Check and limit/clamp value range when converting from f64 to i16
                NumericPredicate::LessThan(value) => {
                    Box::new(aux_track_brief::music_key.lt(value as i16))
                }
                NumericPredicate::LessOrEqual(value) => {
                    Box::new(aux_track_brief::music_key.le(value as i16))
                }
                NumericPredicate::GreaterThan(value) => {
                    Box::new(aux_track_brief::music_key.gt(value as i16))
                }
                NumericPredicate::GreaterOrEqual(value) => {
                    Box::new(aux_track_brief::music_key.ge(value as i16))
                }
                NumericPredicate::Equal(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::music_key.eq(value as i16))
                    } else {
                        Box::new(aux_track_brief::music_key.is_null())
                    }
                }
                NumericPredicate::NotEqual(value) => {
                    if let Some(value) = value {
                        Box::new(aux_track_brief::music_key.ne(value as i16))
                    } else {
                        Box::new(aux_track_brief::music_key.is_not_null())
                    }
                }
            },
        }
    }
}

impl TrackSearchBoxedExpressionBuilder for TagFilter {
    fn build_expression<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(&self);
        match filter_modifier {
            None => Box::new(tbl_track::id.eq_any(subselect)),
            Some(FilterModifier::Complement) => Box::new(tbl_track::id.ne_all(subselect)),
        }
    }
}

impl TrackSearchBoxedExpressionBuilder for MarkerFilter {
    fn build_expression<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        let (subselect, filter_modifier) = select_track_ids_matching_marker_filter(&self);
        match filter_modifier {
            None => Box::new(tbl_track::id.eq_any(subselect)),
            Some(FilterModifier::Complement) => Box::new(tbl_track::id.ne_all(subselect)),
        }
    }
}

impl TrackSearchBoxedExpressionBuilder for TrackSearchFilter {
    fn build_expression<'a>(
        &'a self,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        use crate::api::TrackSearchFilter::*;
        match self {
            Phrase(filter) => filter.build_expression(collection_uid),
            Numeric(filter) => filter.build_expression(collection_uid),
            Tag(filter) => filter.build_expression(collection_uid),
            Marker(filter) => filter.build_expression(collection_uid),
            All(filters) => filters
                .iter()
                .fold(dummy_true_expression(), |expr, filter| {
                    Box::new(expr.and(filter.build_expression(collection_uid)))
                }),
            Any(filters) => filters
                .iter()
                .fold(dummy_false_expression(), |expr, filter| {
                    Box::new(expr.or(filter.build_expression(collection_uid)))
                }),
            Not(filter) => Box::new(diesel::dsl::not(filter.build_expression(collection_uid))),
        }
    }
}
