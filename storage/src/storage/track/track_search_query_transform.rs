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
    Box::new(tbl_track::id.is_not_null())
}

// TODO: replace with "False"
fn dummy_false_expression() -> TrackSearchBoxedExpression<'static> {
    Box::new(tbl_track::id.is_null())
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
                    .any(|target| *target == StringField::SourceUri)
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
                    .any(|target| *target == StringField::ContentType)
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
            // aux_track_brief (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::TrackTitle)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_brief::track_title
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_brief::track_title
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::TrackArtist)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_brief::track_artist
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_brief::track_artist
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::TrackComposer)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_brief::track_composer
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_brief::track_composer
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::AlbumTitle)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_brief::album_title
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_brief::album_title
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::AlbumArtist)
            {
                query = match self.modifier {
                    None => query.or_filter(
                        aux_track_brief::album_artist
                            .like(like_expr.clone())
                            .escape('\\'),
                    ),
                    Some(FilterModifier::Complement) => query.or_filter(
                        aux_track_brief::album_artist
                            .not_like(like_expr.clone())
                            .escape('\\'),
                    ),
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
        match self.field {
            NumericField::ReleaseYear => {
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => query
                            .filter(aux_track_brief::release_year.lt(self.condition.value as i32)),
                        Some(ConditionModifier::Not) => query
                            .filter(aux_track_brief::release_year.ge(self.condition.value as i32)),
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => query
                            .filter(aux_track_brief::release_year.gt(self.condition.value as i32)),
                        Some(ConditionModifier::Not) => query
                            .filter(aux_track_brief::release_year.le(self.condition.value as i32)),
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => query
                            .filter(aux_track_brief::release_year.eq(self.condition.value as i32)),
                        Some(ConditionModifier::Not) => query
                            .filter(aux_track_brief::release_year.ne(self.condition.value as i32)),
                    },
                }
            }
            NumericField::Duration => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => query.filter(aux_track_source::audio_duration.lt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_source::audio_duration.ge(self.condition.value))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => query.filter(aux_track_source::audio_duration.gt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_source::audio_duration.le(self.condition.value))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => query.filter(aux_track_source::audio_duration.eq(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_source::audio_duration.ne(self.condition.value))
                    }
                },
            },
            NumericField::SampleRate => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i32;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => {
                            query.filter(aux_track_source::audio_samplerate.lt(condition_value))
                        }
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_source::audio_samplerate.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => {
                            query.filter(aux_track_source::audio_samplerate.gt(condition_value))
                        }
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_source::audio_samplerate.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => {
                            query.filter(aux_track_source::audio_samplerate.eq(condition_value))
                        }
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_source::audio_samplerate.ne(condition_value))
                        }
                    },
                }
            }
            NumericField::BitRate => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i32;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => query.filter(aux_track_source::audio_bitrate.lt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_source::audio_bitrate.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => query.filter(aux_track_source::audio_bitrate.gt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_source::audio_bitrate.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => query.filter(aux_track_source::audio_bitrate.eq(condition_value)),
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_source::audio_bitrate.ne(condition_value))
                        }
                    },
                }
            }
            NumericField::ChannelCount => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i16;
                match self.condition.comparator {
                    NumericComparator::LessThan => {
                        match self.condition.modifier {
                            None => query
                                .filter(aux_track_source::audio_channel_count.lt(condition_value)),
                            Some(ConditionModifier::Not) => query
                                .filter(aux_track_source::audio_channel_count.ge(condition_value)),
                        }
                    }
                    NumericComparator::GreaterThan => {
                        match self.condition.modifier {
                            None => query
                                .filter(aux_track_source::audio_channel_count.gt(condition_value)),
                            Some(ConditionModifier::Not) => query
                                .filter(aux_track_source::audio_channel_count.le(condition_value)),
                        }
                    }
                    NumericComparator::EqualTo => {
                        match self.condition.modifier {
                            None => query
                                .filter(aux_track_source::audio_channel_count.eq(condition_value)),
                            Some(ConditionModifier::Not) => query
                                .filter(aux_track_source::audio_channel_count.ne(condition_value)),
                        }
                    }
                }
            }
            NumericField::Loudness => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => query.filter(aux_track_source::audio_loudness.lt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_source::audio_loudness.ge(self.condition.value))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => query.filter(aux_track_source::audio_loudness.gt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_source::audio_loudness.le(self.condition.value))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => query.filter(aux_track_source::audio_loudness.eq(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_source::audio_loudness.ne(self.condition.value))
                    }
                },
            },
            NumericField::MusicTempo => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => query.filter(aux_track_brief::music_tempo.lt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_brief::music_tempo.ge(self.condition.value))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => query.filter(aux_track_brief::music_tempo.gt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_brief::music_tempo.le(self.condition.value))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => query.filter(aux_track_brief::music_tempo.eq(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        query.filter(aux_track_brief::music_tempo.ne(self.condition.value))
                    }
                },
            },
            NumericField::MusicKey => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i16;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => query.filter(aux_track_brief::music_key.lt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_brief::music_key.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => query.filter(aux_track_brief::music_key.gt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_brief::music_key.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => query.filter(aux_track_brief::music_key.eq(condition_value)),
                        Some(ConditionModifier::Not) => {
                            query.filter(aux_track_brief::music_key.ne(condition_value))
                        }
                    },
                }
            }
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

        let mut or_expression = dummy_false_expression();
        if !like_expr.is_empty() {
            // aux_track_source (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::SourceUri)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_source::uri_decoded
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_source::uri_decoded
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::ContentType)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_source::content_type
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_source::content_type
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            // aux_track_brief (join)
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::TrackTitle)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_brief::track_title
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_brief::track_title
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::TrackArtist)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_brief::track_artist
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_brief::track_artist
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::TrackComposer)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_brief::track_composer
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_brief::track_composer
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::AlbumTitle)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_brief::album_title
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_brief::album_title
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
            if self.fields.is_empty()
                || self
                    .fields
                    .iter()
                    .any(|target| *target == StringField::AlbumArtist)
            {
                or_expression = match self.modifier {
                    None => Box::new(
                        or_expression.or(aux_track_brief::album_artist
                            .like(like_expr.clone())
                            .escape('\\')),
                    ),
                    Some(FilterModifier::Complement) => Box::new(
                        or_expression.or(aux_track_brief::album_artist
                            .not_like(like_expr.clone())
                            .escape('\\')),
                    ),
                };
            }
        }
        match self.modifier {
            None => or_expression,
            Some(FilterModifier::Complement) => Box::new(diesel::dsl::not(or_expression)),
        }
    }
}

impl TrackSearchBoxedExpressionBuilder for NumericFilter {
    fn build_expression<'a>(
        &'a self,
        _collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedExpression<'a> {
        match self.field {
            NumericField::ReleaseYear => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => Box::new(aux_track_brief::release_year.lt(self.condition.value as i32)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_brief::release_year.ge(self.condition.value as i32))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => Box::new(aux_track_brief::release_year.gt(self.condition.value as i32)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_brief::release_year.le(self.condition.value as i32))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => Box::new(aux_track_brief::release_year.eq(self.condition.value as i32)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_brief::release_year.ne(self.condition.value as i32))
                    }
                },
            },
            NumericField::Duration => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => Box::new(aux_track_source::audio_duration.lt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_source::audio_duration.ge(self.condition.value))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => Box::new(aux_track_source::audio_duration.gt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_source::audio_duration.le(self.condition.value))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => Box::new(aux_track_source::audio_duration.eq(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_source::audio_duration.ne(self.condition.value))
                    }
                },
            },
            NumericField::SampleRate => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i32;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_samplerate.lt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_samplerate.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_samplerate.gt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_samplerate.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_samplerate.eq(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_samplerate.ne(condition_value))
                        }
                    },
                }
            }
            NumericField::BitRate => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i32;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_bitrate.lt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_bitrate.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_bitrate.gt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_bitrate.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_bitrate.eq(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_bitrate.ne(condition_value))
                        }
                    },
                }
            }
            NumericField::ChannelCount => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i16;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_channel_count.lt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_channel_count.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_channel_count.gt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_channel_count.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => Box::new(aux_track_source::audio_channel_count.eq(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_source::audio_channel_count.ne(condition_value))
                        }
                    },
                }
            }
            NumericField::Loudness => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => Box::new(aux_track_source::audio_loudness.lt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_source::audio_loudness.ge(self.condition.value))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => Box::new(aux_track_source::audio_loudness.gt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_source::audio_loudness.le(self.condition.value))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => Box::new(aux_track_source::audio_loudness.eq(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_source::audio_loudness.ne(self.condition.value))
                    }
                },
            },
            NumericField::MusicTempo => match self.condition.comparator {
                NumericComparator::LessThan => match self.condition.modifier {
                    None => Box::new(aux_track_brief::music_tempo.lt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_brief::music_tempo.ge(self.condition.value))
                    }
                },
                NumericComparator::GreaterThan => match self.condition.modifier {
                    None => Box::new(aux_track_brief::music_tempo.gt(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_brief::music_tempo.le(self.condition.value))
                    }
                },
                NumericComparator::EqualTo => match self.condition.modifier {
                    None => Box::new(aux_track_brief::music_tempo.eq(self.condition.value)),
                    Some(ConditionModifier::Not) => {
                        Box::new(aux_track_brief::music_tempo.ne(self.condition.value))
                    }
                },
            },
            NumericField::MusicKey => {
                // TODO: Check value range!
                let condition_value = self.condition.value as i16;
                match self.condition.comparator {
                    NumericComparator::LessThan => match self.condition.modifier {
                        None => Box::new(aux_track_brief::music_key.lt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_brief::music_key.ge(condition_value))
                        }
                    },
                    NumericComparator::GreaterThan => match self.condition.modifier {
                        None => Box::new(aux_track_brief::music_key.gt(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_brief::music_key.le(condition_value))
                        }
                    },
                    NumericComparator::EqualTo => match self.condition.modifier {
                        None => Box::new(aux_track_brief::music_key.eq(condition_value)),
                        Some(ConditionModifier::Not) => {
                            Box::new(aux_track_brief::music_key.ne(condition_value))
                        }
                    },
                }
            }
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
        }
    }
}

impl TrackSearchQueryTransform for TrackSearchFilter {
    fn apply_to_query<'a>(
        &'a self,
        query: TrackSearchBoxedQuery<'a>,
        collection_uid: Option<&EntityUid>,
    ) -> TrackSearchBoxedQuery<'a> {
        query.filter(self.build_expression(collection_uid))
    }
}
