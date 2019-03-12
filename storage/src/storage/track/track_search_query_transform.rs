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
        diesel::sql_types::Timestamp,
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
                            diesel::query_source::joins::JoinOn<
                                diesel::query_source::joins::Join<
                                    tbl_track::table,
                                    aux_track_overview::table,
                                    diesel::query_source::joins::Inner,
                                >,
                                diesel::expression::operators::Eq<
                                    diesel::expression::nullable::Nullable<
                                        aux_track_overview::columns::track_id,
                                    >,
                                    diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                                >,
                            >,
                            aux_track_summary::table,
                            diesel::query_source::joins::Inner,
                        >,
                        diesel::expression::operators::Eq<
                            diesel::expression::nullable::Nullable<
                                aux_track_summary::columns::track_id,
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
                        diesel::query_source::joins::JoinOn<
                            diesel::query_source::joins::Join<
                                tbl_track::table,
                                aux_track_overview::table,
                                diesel::query_source::joins::Inner,
                            >,
                            diesel::expression::operators::Eq<
                                diesel::expression::nullable::Nullable<
                                    aux_track_overview::columns::track_id,
                                >,
                                diesel::expression::nullable::Nullable<tbl_track::columns::id>,
                            >,
                        >,
                        aux_track_summary::table,
                        diesel::query_source::joins::Inner,
                    >,
                    diesel::expression::operators::Eq<
                        diesel::expression::nullable::Nullable<
                            aux_track_summary::columns::track_id,
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
>;

type TrackSearchBoxedExpression<'a> = Box<
    BoxableExpression<TrackSearchQuery, diesel::sqlite::Sqlite, SqlType = diesel::sql_types::Bool>
        + 'a,
>;

// TODO: replace with "False"
fn dummy_expression() -> TrackSearchBoxedExpression<'static> {
    Box::new(tbl_track::id.is_null().and(tbl_track::id.is_not_null()))
}

pub trait AsTrackSearchQueryExpression {
    fn predicate<'a>(
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

impl TrackSearchQueryTransform for PhraseFilter {
    fn apply_to_query<'a>(
        &'a self,
        mut query: TrackSearchBoxedQuery<'a>,
        _: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        // Escape wildcard character with backslash (see below)
        let escaped = self.phrase.replace('\\', "\\\\").replace('%', "\\%");
        let escaped_and_tokenized = escaped.split_whitespace().filter(|token| !token.is_empty());
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
            // aux_track_source (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::SourceUri)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_source::uri_decoded
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_source::uri_decoded
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::SourceType)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_source::content_type
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_source::content_type
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }

            // aux_track_overview (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::TrackTitle)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_overview::track_title
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_overview::track_title
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::AlbumTitle)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_overview::album_title
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_overview::album_title
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }

            // aux_track_summary (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::TrackArtist)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_summary::track_artist
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_summary::track_artist
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::AlbumArtist)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_summary::album_artist
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_summary::album_artist
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }

            // aux_track_comment (subselect)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::Comments)
            {
                let subselect = aux_track_comment::table
                    .select(aux_track_comment::track_id)
                    .filter(aux_track_comment::text.like(like_expr.clone()).escape('\\'));
                query = match self.modifier {
                    None => query.or_filter(tbl_track::id.eq_any(subselect)),
                    Some(FilterModifier::Complement) => {
                        query.or_filter(tbl_track::id.ne_all(subselect))
                    }
                };
            }
        }
        query
    }
}

impl TrackSearchQueryTransform for NumericFilter {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        _: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        match select_track_ids_from_profile_matching_numeric_filter(self) {
            Some((subselect, filter_modifier)) => match filter_modifier {
                None => query.filter(tbl_track::id.eq_any(subselect)),
                Some(FilterModifier::Complement) => query.filter(tbl_track::id.ne_all(subselect)),
            },
            None => match self.field {
                NumericField::DurationMs => {
                    match self.condition.comparator {
                        NumericComparator::LessThan => match self.condition.modifier {
                            None => match self.modifier {
                                None => query.filter(
                                    aux_track_source::audio_duration_ms.lt(self.condition.value),
                                ),
                                Some(FilterModifier::Complement) => query
                                    .filter(not(aux_track_source::audio_duration_ms
                                        .lt(self.condition.value))),
                            },
                            Some(ConditionModifier::Not) => match self.modifier {
                                None => query.filter(
                                    aux_track_source::audio_duration_ms.ge(self.condition.value),
                                ),
                                Some(FilterModifier::Complement) => query
                                    .filter(not(aux_track_source::audio_duration_ms
                                        .ge(self.condition.value))),
                            },
                        },
                        NumericComparator::GreaterThan => match self.condition.modifier {
                            None => match self.modifier {
                                None => query.filter(
                                    aux_track_source::audio_duration_ms.gt(self.condition.value),
                                ),
                                Some(FilterModifier::Complement) => query
                                    .filter(not(aux_track_source::audio_duration_ms
                                        .gt(self.condition.value))),
                            },
                            Some(ConditionModifier::Not) => match self.modifier {
                                None => query.filter(
                                    aux_track_source::audio_duration_ms.le(self.condition.value),
                                ),
                                Some(FilterModifier::Complement) => query
                                    .filter(not(aux_track_source::audio_duration_ms
                                        .le(self.condition.value))),
                            },
                        },
                        NumericComparator::EqualTo => match self.condition.modifier {
                            None => match self.modifier {
                                None => query.filter(
                                    aux_track_source::audio_duration_ms.eq(self.condition.value),
                                ),
                                Some(FilterModifier::Complement) => query
                                    .filter(not(aux_track_source::audio_duration_ms
                                        .eq(self.condition.value))),
                            },
                            Some(ConditionModifier::Not) => match self.modifier {
                                None => query.filter(
                                    aux_track_source::audio_duration_ms.ne(self.condition.value),
                                ),
                                Some(FilterModifier::Complement) => query
                                    .filter(not(aux_track_source::audio_duration_ms
                                        .ne(self.condition.value))),
                            },
                        },
                    }
                }
                NumericField::SampleRateHz => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .lt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .lt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .ge(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .ge(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .gt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .gt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .le(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .le(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .eq(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .eq(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_samplerate_hz
                                    .ne(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_samplerate_hz
                                    .ne(self.condition.value as i32))),
                        },
                    },
                },
                NumericField::BitRateBps => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.lt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .lt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.ge(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .ge(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.gt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .gt(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.le(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .le(self.condition.value as i32))),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.eq(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .eq(self.condition.value as i32))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_bitrate_bps.ne(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_bitrate_bps
                                    .ne(self.condition.value as i32))),
                        },
                    },
                },
                NumericField::ChannelsCount => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .lt(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .lt(self.condition.value as i16))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .ge(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .ge(self.condition.value as i16))),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .gt(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .gt(self.condition.value as i16))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .le(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .le(self.condition.value as i16))),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .eq(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .eq(self.condition.value as i16))),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => query.filter(
                                aux_track_source::audio_channels_count
                                    .ne(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => query
                                .filter(not(aux_track_source::audio_channels_count
                                    .ne(self.condition.value as i16))),
                        },
                    },
                },
                numeric_field => {
                    unreachable!("unhandled numeric filter field: {:?}", numeric_field)
                }
            },
        }
    }
}

impl TrackSearchQueryTransform for TagFilter {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        _: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(&self);
        match filter_modifier {
            None => query.filter(tbl_track::id.eq_any(subselect)),
            Some(FilterModifier::Complement) => query.filter(tbl_track::id.ne_all(subselect)),
        }
    }
}

impl TrackSearchQueryTransform for TrackSort {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        let direction = self
            .direction
            .unwrap_or_else(|| TrackSort::default_direction(self.field));
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
                SortDirection::Ascending => query.then_order_by(tbl_track::rev_timestamp.asc()),
                SortDirection::Descending => query.then_order_by(tbl_track::rev_timestamp.desc()),
            },
            TrackSortField::TrackTitle => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::track_title.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::track_title.desc())
                }
            },
            TrackSortField::AlbumTitle => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::album_title.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::album_title.desc())
                }
            },
            TrackSortField::ReleasedAt => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::released_at.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::released_at.desc())
                }
            },
            TrackSortField::ReleasedBy => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_overview::released_by.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_overview::released_by.desc())
                }
            },
            TrackSortField::TrackArtist => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_summary::track_artist.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_summary::track_artist.desc())
                }
            },
            TrackSortField::AlbumArtist => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(aux_track_summary::album_artist.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(aux_track_summary::album_artist.desc())
                }
            },
        }
    }
}

impl AsTrackSearchQueryExpression for PhraseFilter {
    fn predicate<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        // Escape wildcard character with backslash (see below)
        let escaped = self.phrase.replace('\\', "\\\\").replace('%', "\\%");
        let escaped_and_tokenized = escaped.split_whitespace().filter(|token| !token.is_empty());
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

        let mut expression = dummy_expression();

        if !like_expr.is_empty() {
            // aux_track_source (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::SourceUri)
            {
                expression = match self.modifier {
                    None => Box::new(
                        expression.or(aux_track_source::uri_decoded
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        expression.or(aux_track_source::uri_decoded
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::SourceType)
            {
                expression = match self.modifier {
                    None => Box::new(
                        expression.or(aux_track_source::content_type
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        expression.or(aux_track_source::content_type
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }

            // aux_track_overview (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::TrackTitle)
            {
                expression = match self.modifier {
                    None => Box::new(
                        expression.or(aux_track_overview::track_title
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        expression.or(aux_track_overview::track_title
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::AlbumTitle)
            {
                expression = match self.modifier {
                    None => Box::new(
                        expression.or(aux_track_overview::album_title
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        expression.or(aux_track_overview::album_title
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }

            // aux_track_summary (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::TrackArtist)
            {
                expression = match self.modifier {
                    None => Box::new(
                        expression.or(aux_track_summary::track_artist
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        expression.or(aux_track_summary::track_artist
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::AlbumArtist)
            {
                expression = match self.modifier {
                    None => Box::new(
                        expression.or(aux_track_summary::album_artist
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        expression.or(aux_track_summary::album_artist
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }

            // aux_track_comment (subselect)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == PhraseField::Comments)
            {
                let subselect = aux_track_comment::table
                    .select(aux_track_comment::track_id)
                    .filter(aux_track_comment::text.like(like_expr.clone()).escape('\\'));
                expression = match self.modifier {
                    None => Box::new(expression.or(tbl_track::id.eq_any(subselect))),
                    Some(FilterModifier::Complement) => {
                        Box::new(expression.or(tbl_track::id.ne_all(subselect)))
                    }
                };
            }
        }
        expression
    }
}

impl AsTrackSearchQueryExpression for NumericFilter {
    fn predicate<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        match select_track_ids_from_profile_matching_numeric_filter(self) {
            Some((subselect, filter_modifier)) => match filter_modifier {
                None => Box::new(tbl_track::id.eq_any(subselect)),
                Some(FilterModifier::Complement) => Box::new(tbl_track::id.ne_all(subselect)),
            },
            None => match self.field {
                NumericField::DurationMs => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_duration_ms.lt(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => Box::new(not(
                                aux_track_source::audio_duration_ms.lt(self.condition.value),
                            )),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_duration_ms.ge(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => Box::new(not(
                                aux_track_source::audio_duration_ms.ge(self.condition.value),
                            )),
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_duration_ms.gt(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => Box::new(not(
                                aux_track_source::audio_duration_ms.gt(self.condition.value),
                            )),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_duration_ms.le(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => Box::new(not(
                                aux_track_source::audio_duration_ms.le(self.condition.value),
                            )),
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_duration_ms.eq(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => Box::new(not(
                                aux_track_source::audio_duration_ms.eq(self.condition.value),
                            )),
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_duration_ms.ne(self.condition.value),
                            ),
                            Some(FilterModifier::Complement) => Box::new(not(
                                aux_track_source::audio_duration_ms.ne(self.condition.value),
                            )),
                        },
                    },
                },
                NumericField::SampleRateHz => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_samplerate_hz
                                    .lt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_samplerate_hz
                                    .lt(self.condition.value as i32)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_samplerate_hz
                                    .ge(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_samplerate_hz
                                    .ge(self.condition.value as i32)))
                            }
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_samplerate_hz
                                    .gt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_samplerate_hz
                                    .gt(self.condition.value as i32)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_samplerate_hz
                                    .le(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_samplerate_hz
                                    .le(self.condition.value as i32)))
                            }
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_samplerate_hz
                                    .eq(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_samplerate_hz
                                    .eq(self.condition.value as i32)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_samplerate_hz
                                    .ne(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_samplerate_hz
                                    .ne(self.condition.value as i32)))
                            }
                        },
                    },
                },
                NumericField::BitRateBps => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_bitrate_bps.lt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_bitrate_bps
                                    .lt(self.condition.value as i32)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_bitrate_bps.ge(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_bitrate_bps
                                    .ge(self.condition.value as i32)))
                            }
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_bitrate_bps.gt(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_bitrate_bps
                                    .gt(self.condition.value as i32)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_bitrate_bps.le(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_bitrate_bps
                                    .le(self.condition.value as i32)))
                            }
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_bitrate_bps.eq(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_bitrate_bps
                                    .eq(self.condition.value as i32)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_bitrate_bps.ne(self.condition.value as i32),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_bitrate_bps
                                    .ne(self.condition.value as i32)))
                            }
                        },
                    },
                },
                NumericField::ChannelsCount => match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_channels_count
                                    .lt(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_channels_count
                                    .lt(self.condition.value as i16)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_channels_count
                                    .ge(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_channels_count
                                    .ge(self.condition.value as i16)))
                            }
                        },
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_channels_count
                                    .gt(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_channels_count
                                    .gt(self.condition.value as i16)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_channels_count
                                    .le(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_channels_count
                                    .le(self.condition.value as i16)))
                            }
                        },
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_channels_count
                                    .eq(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_channels_count
                                    .eq(self.condition.value as i16)))
                            }
                        },
                        Some(ConditionModifier::Not) => match self.modifier {
                            None => Box::new(
                                aux_track_source::audio_channels_count
                                    .ne(self.condition.value as i16),
                            ),
                            Some(FilterModifier::Complement) => {
                                Box::new(not(aux_track_source::audio_channels_count
                                    .ne(self.condition.value as i16)))
                            }
                        },
                    },
                },
                numeric_field => {
                    unreachable!("unhandled numeric filter field: {:?}", numeric_field)
                }
            },
        }
    }
}

impl AsTrackSearchQueryExpression for TagFilter {
    fn predicate<'a>(
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

impl AsTrackSearchQueryExpression for TrackSearchFilterPredicate {
    fn predicate<'a>(
        &'a self,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        use api::TrackSearchFilterPredicate::*;
        match self {
            PhraseFilter(filter) => filter.predicate(collection_uid),
            NumericFilter(filter) => filter.predicate(collection_uid),
            TagFilter(filter) => filter.predicate(collection_uid),
            And(predicate_vec) => {
                if let Some(first_predicate) = predicate_vec.first() {
                    let mut expression = first_predicate.predicate(collection_uid);
                    for predicate in predicate_vec {
                        expression = Box::new(expression.and(predicate.predicate(collection_uid)));
                    }
                    expression
                } else {
                    dummy_expression()
                }
            }
            Or(predicate_vec) => {
                if let Some(first_predicate) = predicate_vec.first() {
                    let mut expression = first_predicate.predicate(collection_uid);
                    for predicate in predicate_vec {
                        expression = Box::new(expression.or(predicate.predicate(collection_uid)));
                    }
                    expression
                } else {
                    dummy_expression()
                }
            }
        }
    }
}

impl TrackSearchQueryTransform for TrackSearchFilterPredicate {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        query.filter(self.predicate(collection_uid))
    }
}
