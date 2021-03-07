// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::{
    db::{
        media_source::schema::*, media_tracker::schema::*, track::schema::*, track_cue::schema::*,
        track_tag::schema::*,
    },
    prelude::*,
};

use aoide_core::util::clock::YYYYMMDD;

use aoide_repo::{
    tag::Filter as TagFilter,
    track::{
        ConditionFilter, DateTimeField, DateTimeFieldFilter, NumericField, NumericFieldFilter,
        PhraseFieldFilter, SearchFilter, SortField, SortOrder, StringField,
    },
};

use diesel::{
    query_source::joins as diesel_joins,
    sql_types::{BigInt, Binary, Double, Integer, Nullable, SmallInt, Text},
    BoolExpressionMethods, BoxableExpression, ExpressionMethods, TextExpressionMethods,
};

///////////////////////////////////////////////////////////////////////

// The following type expression has been copied from a compiler error message ;)
type TrackSearchQuery = diesel_joins::JoinOn<
    diesel_joins::Join<track::table, media_source::table, diesel_joins::Inner>,
    diesel::expression::operators::Eq<
        diesel::expression::nullable::Nullable<track::columns::media_source_id>,
        diesel::expression::nullable::Nullable<media_source::columns::row_id>,
    >,
>;

// The following type expression has been copied from a compiler error message ;)
type TrackSearchBoxedQuery<'a> = diesel::query_builder::BoxedSelectStatement<
    'a,
    (
        // track
        //(
        BigInt,
        BigInt,
        BigInt,
        Binary,
        BigInt,
        BigInt,
        Nullable<Text>,
        Nullable<BigInt>,
        Nullable<Integer>,
        Nullable<Text>,
        Nullable<Text>,
        SmallInt,
        Nullable<SmallInt>,
        Nullable<SmallInt>,
        Nullable<SmallInt>,
        Nullable<SmallInt>,
        Nullable<SmallInt>,
        Nullable<SmallInt>,
        Nullable<Double>,
        SmallInt,
        Nullable<SmallInt>,
        Nullable<SmallInt>,
        SmallInt,
        Nullable<Integer>,
        Nullable<SmallInt>,
        Nullable<Text>,
        Nullable<BigInt>,
        Nullable<BigInt>,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        Nullable<Text>,
        //),
        /*
        // media_source
        (
            ...add when needed...
        ),
        */
    ),
    TrackSearchQuery,
    diesel::sqlite::Sqlite,
>;

type TrackSearchBoxedExpression<'a> = Box<
    dyn BoxableExpression<
            TrackSearchQuery,
            diesel::sqlite::Sqlite,
            SqlType = diesel::sql_types::Bool,
        > + 'a,
>;

// TODO: replace with "True"
fn dummy_true_expression() -> TrackSearchBoxedExpression<'static> {
    Box::new(track::row_id.is_not_null()) // always true
}

// TODO: replace with "False"
fn dummy_false_expression() -> TrackSearchBoxedExpression<'static> {
    Box::new(track::row_id.is_null()) // always false
}

pub trait TrackSearchBoxedExpressionBuilder {
    fn build_expression(&self) -> TrackSearchBoxedExpression<'_>;
}

pub trait TrackSearchQueryTransform {
    fn apply_to_query<'a>(&'a self, query: TrackSearchBoxedQuery<'a>) -> TrackSearchBoxedQuery<'a>;
}

impl TrackSearchQueryTransform for SortOrder {
    fn apply_to_query<'a>(&'a self, query: TrackSearchBoxedQuery<'a>) -> TrackSearchBoxedQuery<'a> {
        let direction = self.direction;
        match self.field {
            SortField::AlbumArtist => match direction {
                SortDirection::Ascending => query.then_order_by(track::aux_track_artist.asc()),
                SortDirection::Descending => query.then_order_by(track::aux_album_artist.desc()),
            },
            SortField::AlbumTitle => match direction {
                SortDirection::Ascending => query.then_order_by(track::aux_album_title.asc()),
                SortDirection::Descending => query.then_order_by(track::aux_album_title.desc()),
            },
            SortField::AudioBitrateBps => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::audio_bitrate_bps.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::audio_bitrate_bps.desc())
                }
            },
            SortField::AudioChannelCount => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::audio_channel_count.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::audio_channel_count.desc())
                }
            },
            SortField::AudioDurationMs => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::audio_duration_ms.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::audio_duration_ms.desc())
                }
            },
            SortField::AudioLoudnessLufs => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::audio_loudness_lufs.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::audio_loudness_lufs.desc())
                }
            },
            SortField::AudioSampleRateHz => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::audio_samplerate_hz.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::audio_samplerate_hz.desc())
                }
            },
            SortField::CreatedAt => match direction {
                SortDirection::Ascending => query.then_order_by(track::row_created_ms.asc()),
                SortDirection::Descending => query.then_order_by(track::row_created_ms.desc()),
            },
            SortField::DiscNumber => match direction {
                SortDirection::Ascending => query.then_order_by(track::disc_number.asc()),
                SortDirection::Descending => query.then_order_by(track::disc_number.desc()),
            },
            SortField::DiscTotal => match direction {
                SortDirection::Ascending => query.then_order_by(track::disc_total.asc()),
                SortDirection::Descending => query.then_order_by(track::disc_total.desc()),
            },
            SortField::LastPlayedAt => match direction {
                SortDirection::Ascending => query.then_order_by(track::last_played_ms.asc()),
                SortDirection::Descending => query.then_order_by(track::last_played_ms.desc()),
            },
            SortField::MusicTempoBpm => match direction {
                SortDirection::Ascending => query.then_order_by(track::music_tempo_bpm.asc()),
                SortDirection::Descending => query.then_order_by(track::music_tempo_bpm.desc()),
            },
            SortField::MusicKeyCode => match direction {
                SortDirection::Ascending => query.then_order_by(track::music_key_code.asc()),
                SortDirection::Descending => query.then_order_by(track::music_key_code.desc()),
            },
            SortField::ReleaseDate => match direction {
                SortDirection::Ascending => query.then_order_by(track::released_at_yyyymmdd.asc()),
                SortDirection::Descending => {
                    query.then_order_by(track::released_at_yyyymmdd.desc())
                }
            },
            SortField::SourceCollectedAt => match direction {
                SortDirection::Ascending => query.then_order_by(media_source::collected_ms.asc()),
                SortDirection::Descending => query.then_order_by(media_source::collected_ms.desc()),
            },
            SortField::SourcePath => match direction {
                SortDirection::Ascending => query.then_order_by(media_source::path.asc()),
                SortDirection::Descending => query.then_order_by(media_source::path.desc()),
            },
            SortField::SourceType => match direction {
                SortDirection::Ascending => query.then_order_by(media_source::content_type.asc()),
                SortDirection::Descending => query.then_order_by(media_source::content_type.desc()),
            },
            SortField::TimesPlayed => match direction {
                SortDirection::Ascending => query.then_order_by(track::times_played.asc()),
                SortDirection::Descending => query.then_order_by(track::times_played.desc()),
            },
            SortField::TrackArtist => match direction {
                SortDirection::Ascending => query.then_order_by(track::aux_track_artist.asc()),
                SortDirection::Descending => query.then_order_by(track::aux_track_artist.desc()),
            },
            SortField::TrackNumber => match direction {
                SortDirection::Ascending => query.then_order_by(track::track_number.asc()),
                SortDirection::Descending => query.then_order_by(track::track_number.desc()),
            },
            SortField::TrackTitle => match direction {
                SortDirection::Ascending => query.then_order_by(track::aux_track_title.asc()),
                SortDirection::Descending => query.then_order_by(track::aux_track_title.desc()),
            },
            SortField::TrackTotal => match direction {
                SortDirection::Ascending => query.then_order_by(track::track_total.asc()),
                SortDirection::Descending => query.then_order_by(track::track_total.desc()),
            },
            SortField::SourceSynchronizedAt => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::synchronized_ms.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::synchronized_ms.desc())
                }
            },
            SortField::UpdatedAt => match direction {
                SortDirection::Ascending => query.then_order_by(track::row_updated_ms.asc()),
                SortDirection::Descending => query.then_order_by(track::row_updated_ms.desc()),
            },
        }
    }
}

fn build_phrase_field_filter_expression(
    filter: &PhraseFieldFilter,
) -> TrackSearchBoxedExpression<'_> {
    // Escape wildcard character with backslash (see below)
    let escaped_terms: Vec<_> = filter
        .terms
        .iter()
        .map(|t| escape_like_matches(t))
        .collect();
    let escaped_terms_str_len = escaped_terms.iter().fold(0, |len, term| len + term.len());
    // TODO: Use Rc<String> to avoid cloning strings?
    let like_expr = if escaped_terms_str_len > 0 {
        let mut like_expr = escaped_terms.iter().fold(
            String::with_capacity(escaped_terms_str_len + escaped_terms.len() + 1),
            |mut like_expr, term| {
                // Prepend wildcard character before each part
                like_expr.push(LIKE_WILDCARD_CHARACTER);
                like_expr.push_str(term);
                like_expr
            },
        );
        // Append final wildcard character after last part
        like_expr.push(LIKE_WILDCARD_CHARACTER);
        like_expr
    } else {
        // unused
        String::new()
    };

    let mut or_expression = dummy_false_expression();
    // media_source (join)
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::SourcePath)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(media_source::path.is_null())
                    .or(media_source::path.eq(String::default())),
            )
        } else {
            Box::new(or_expression.or(media_source::path.like(like_expr.clone()).escape('\\')))
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::SourceType)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(media_source::content_type.is_null())
                    .or(media_source::content_type.eq(String::default())),
            )
        } else {
            Box::new(
                or_expression.or(media_source::content_type
                    .like(like_expr.clone())
                    .escape('\\')),
            )
        };
    }
    // track (join)
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::TrackTitle)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(track::aux_track_title.is_null())
                    .or(track::aux_track_title.eq(String::default())),
            )
        } else {
            Box::new(or_expression.or(track::aux_track_title.like(like_expr.clone()).escape('\\')))
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::TrackArtist)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(track::aux_track_artist.is_null())
                    .or(track::aux_track_artist.eq(String::default())),
            )
        } else {
            Box::new(or_expression.or(track::aux_track_artist.like(like_expr.clone()).escape('\\')))
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::TrackComposer)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(track::aux_track_composer.is_null())
                    .or(track::aux_track_composer.eq(String::default())),
            )
        } else {
            Box::new(
                or_expression.or(track::aux_track_composer
                    .like(like_expr.clone())
                    .escape('\\')),
            )
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::AlbumTitle)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(track::aux_album_title.is_null())
                    .or(track::aux_album_title.eq(String::default())),
            )
        } else {
            Box::new(or_expression.or(track::aux_album_title.like(like_expr.clone()).escape('\\')))
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::AlbumArtist)
    {
        or_expression = if like_expr.is_empty() {
            Box::new(
                or_expression
                    .or(track::aux_album_artist.is_null())
                    .or(track::aux_album_artist.eq(String::default())),
            )
        } else {
            Box::new(or_expression.or(track::aux_album_artist.like(like_expr).escape('\\')))
        };
    }
    or_expression
}

fn build_numeric_field_filter_expression(
    filter: &NumericFieldFilter,
) -> TrackSearchBoxedExpression<'_> {
    use NumericField::*;
    use ScalarPredicate::*;
    match filter.field {
        AudioDurationMs => match filter.predicate {
            LessThan(value) => Box::new(media_source::audio_duration_ms.lt(value)),
            LessOrEqual(value) => Box::new(media_source::audio_duration_ms.le(value)),
            GreaterThan(value) => Box::new(media_source::audio_duration_ms.gt(value)),
            GreaterOrEqual(value) => Box::new(media_source::audio_duration_ms.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_duration_ms.eq(value))
                } else {
                    Box::new(media_source::audio_duration_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_duration_ms.ne(value))
                } else {
                    Box::new(media_source::audio_duration_ms.is_not_null())
                }
            }
        },
        AudioSampleRateHz => match filter.predicate {
            LessThan(value) => Box::new(media_source::audio_samplerate_hz.lt(value)),
            LessOrEqual(value) => Box::new(media_source::audio_samplerate_hz.le(value)),
            GreaterThan(value) => Box::new(media_source::audio_samplerate_hz.gt(value)),
            GreaterOrEqual(value) => Box::new(media_source::audio_samplerate_hz.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_samplerate_hz.eq(value))
                } else {
                    Box::new(media_source::audio_samplerate_hz.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_samplerate_hz.ne(value))
                } else {
                    Box::new(media_source::audio_samplerate_hz.is_not_null())
                }
            }
        },
        AudioBitrateBps => match filter.predicate {
            LessThan(value) => Box::new(media_source::audio_bitrate_bps.lt(value)),
            LessOrEqual(value) => Box::new(media_source::audio_bitrate_bps.le(value)),
            GreaterThan(value) => Box::new(media_source::audio_bitrate_bps.gt(value)),
            GreaterOrEqual(value) => Box::new(media_source::audio_bitrate_bps.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_bitrate_bps.eq(value))
                } else {
                    Box::new(media_source::audio_bitrate_bps.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_bitrate_bps.ne(value))
                } else {
                    Box::new(media_source::audio_bitrate_bps.is_not_null())
                }
            }
        },
        AudioChannelCount => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(media_source::audio_channel_count.lt(value as i16)),
            LessOrEqual(value) => Box::new(media_source::audio_channel_count.le(value as i16)),
            GreaterThan(value) => Box::new(media_source::audio_channel_count.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(media_source::audio_channel_count.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_channel_count.eq(value as i16))
                } else {
                    Box::new(media_source::audio_channel_count.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_channel_count.ne(value as i16))
                } else {
                    Box::new(media_source::audio_channel_count.is_not_null())
                }
            }
        },
        AudioLoudnessLufs => match filter.predicate {
            LessThan(value) => Box::new(media_source::audio_loudness_lufs.lt(value)),
            LessOrEqual(value) => Box::new(media_source::audio_loudness_lufs.le(value)),
            GreaterThan(value) => Box::new(media_source::audio_loudness_lufs.gt(value)),
            GreaterOrEqual(value) => Box::new(media_source::audio_loudness_lufs.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_loudness_lufs.eq(value))
                } else {
                    Box::new(media_source::audio_loudness_lufs.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::audio_loudness_lufs.ne(value))
                } else {
                    Box::new(media_source::audio_loudness_lufs.is_not_null())
                }
            }
        },
        TrackNumber => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(track::track_number.lt(value as i16)),
            LessOrEqual(value) => Box::new(track::track_number.le(value as i16)),
            GreaterThan(value) => Box::new(track::track_number.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(track::track_number.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::track_number.eq(value as i16))
                } else {
                    Box::new(track::track_number.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::track_number.ne(value as i16))
                } else {
                    Box::new(track::track_number.is_not_null())
                }
            }
        },
        TrackTotal => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(track::track_total.lt(value as i16)),
            LessOrEqual(value) => Box::new(track::track_total.le(value as i16)),
            GreaterThan(value) => Box::new(track::track_total.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(track::track_total.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::track_total.eq(value as i16))
                } else {
                    Box::new(track::track_total.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::track_total.ne(value as i16))
                } else {
                    Box::new(track::track_total.is_not_null())
                }
            }
        },
        DiscNumber => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(track::disc_number.lt(value as i16)),
            LessOrEqual(value) => Box::new(track::disc_number.le(value as i16)),
            GreaterThan(value) => Box::new(track::disc_number.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(track::disc_number.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::disc_number.eq(value as i16))
                } else {
                    Box::new(track::disc_number.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::disc_number.ne(value as i16))
                } else {
                    Box::new(track::disc_number.is_not_null())
                }
            }
        },
        DiscTotal => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(track::disc_total.lt(value as i16)),
            LessOrEqual(value) => Box::new(track::disc_total.le(value as i16)),
            GreaterThan(value) => Box::new(track::disc_total.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(track::disc_total.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::disc_total.eq(value as i16))
                } else {
                    Box::new(track::disc_total.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::disc_total.ne(value as i16))
                } else {
                    Box::new(track::disc_total.is_not_null())
                }
            }
        },
        ReleaseDate => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to YYYYMMDD
            LessThan(value) => Box::new(track::released_at_yyyymmdd.lt(value as YYYYMMDD)),
            LessOrEqual(value) => Box::new(track::released_at_yyyymmdd.le(value as YYYYMMDD)),
            GreaterThan(value) => Box::new(track::released_at_yyyymmdd.gt(value as YYYYMMDD)),
            GreaterOrEqual(value) => Box::new(track::released_at_yyyymmdd.ge(value as YYYYMMDD)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::released_at_yyyymmdd.eq(value as YYYYMMDD))
                } else {
                    Box::new(track::released_at_yyyymmdd.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::released_at_yyyymmdd.ne(value as YYYYMMDD))
                } else {
                    Box::new(track::released_at_yyyymmdd.is_not_null())
                }
            }
        },
        MusicTempoBpm => match filter.predicate {
            LessThan(value) => Box::new(track::music_tempo_bpm.lt(value)),
            LessOrEqual(value) => Box::new(track::music_tempo_bpm.le(value)),
            GreaterThan(value) => Box::new(track::music_tempo_bpm.gt(value)),
            GreaterOrEqual(value) => Box::new(track::music_tempo_bpm.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::music_tempo_bpm.eq(value))
                } else {
                    Box::new(track::music_tempo_bpm.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::music_tempo_bpm.ne(value))
                } else {
                    Box::new(track::music_tempo_bpm.is_not_null())
                }
            }
        },
        MusicKeyCode => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(track::music_key_code.lt(value as i16)),
            LessOrEqual(value) => Box::new(track::music_key_code.le(value as i16)),
            GreaterThan(value) => Box::new(track::music_key_code.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(track::music_key_code.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::music_key_code.eq(value as i16))
                } else {
                    Box::new(track::music_key_code.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::music_key_code.ne(value as i16))
                } else {
                    Box::new(track::music_key_code.is_not_null())
                }
            }
        },
    }
}

fn build_datetime_field_filter_expression(
    filter: &DateTimeFieldFilter,
) -> TrackSearchBoxedExpression<'_> {
    use DateTimeField::*;
    use ScalarPredicate::*;
    match filter.field {
        LastPlayedAt => match filter.predicate {
            LessThan(value) => Box::new(track::last_played_ms.lt(value.timestamp_millis())),
            LessOrEqual(value) => Box::new(track::last_played_ms.le(value.timestamp_millis())),
            GreaterThan(value) => Box::new(track::last_played_ms.gt(value.timestamp_millis())),
            GreaterOrEqual(value) => Box::new(track::last_played_ms.ge(value.timestamp_millis())),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::last_played_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(track::last_played_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::last_played_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(track::last_played_ms.is_not_null())
                }
            }
        },
        ReleasedAt => match filter.predicate {
            LessThan(value) => Box::new(track::released_ms.lt(value.timestamp_millis())),
            LessOrEqual(value) => Box::new(track::released_ms.le(value.timestamp_millis())),
            GreaterThan(value) => Box::new(track::released_ms.gt(value.timestamp_millis())),
            GreaterOrEqual(value) => Box::new(track::released_ms.ge(value.timestamp_millis())),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::released_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(track::released_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::released_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(track::released_ms.is_not_null())
                }
            }
        },
        SourceCollectedAt => match filter.predicate {
            LessThan(value) => Box::new(media_source::collected_ms.lt(value.timestamp_millis())),
            LessOrEqual(value) => Box::new(media_source::collected_ms.le(value.timestamp_millis())),
            GreaterThan(value) => Box::new(media_source::collected_ms.gt(value.timestamp_millis())),
            GreaterOrEqual(value) => {
                Box::new(media_source::collected_ms.ge(value.timestamp_millis()))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::collected_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(media_source::collected_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::collected_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(media_source::collected_ms.is_not_null())
                }
            }
        },
        SourceSynchronizedAt => match filter.predicate {
            LessThan(value) => Box::new(media_source::synchronized_ms.lt(value.timestamp_millis())),
            LessOrEqual(value) => {
                Box::new(media_source::synchronized_ms.le(value.timestamp_millis()))
            }
            GreaterThan(value) => {
                Box::new(media_source::synchronized_ms.gt(value.timestamp_millis()))
            }
            GreaterOrEqual(value) => {
                Box::new(media_source::synchronized_ms.ge(value.timestamp_millis()))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::synchronized_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(media_source::synchronized_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::synchronized_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(media_source::synchronized_ms.is_not_null())
                }
            }
        },
    }
}

fn build_condition_filter_expression(
    filter: ConditionFilter,
) -> TrackSearchBoxedExpression<'static> {
    use ConditionFilter::*;
    match filter {
        SourceTracked => Box::new(
            media_source::row_id
                .eq_any(media_tracker_source::table.select(media_tracker_source::source_id)),
        ),
        SourceUntracked => Box::new(
            media_source::row_id
                .ne_all(media_tracker_source::table.select(media_tracker_source::source_id)),
        ),
    }
}

fn select_track_ids_matching_tag_filter<'a, DB>(
    tag_filter: &'a TagFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        track_tag::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = track_tag::table.select(track_tag::track_id).into_boxed();

    // Filter facet(s)
    if let Some(ref facets) = tag_filter.facets {
        if facets.is_empty() {
            // unfaceted tags without a facet
            select = select.filter(track_tag::facet.is_null());
        } else {
            // tags with any of the given facets
            select = select.filter(track_tag::facet.eq_any(facets));
        }
    }

    // Filter labels
    if let Some(ref label) = tag_filter.label {
        let (cmp, val, dir) = label.borrow().into();
        let string_cmp_op = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => StringCmpOp::Equal(val.to_owned()),
            StringCompare::Prefix => StringCmpOp::Prefix(escape_single_quotes(val), val.len()),
            // Like comparisons with escaped wildcard character
            StringCompare::StartsWith => StringCmpOp::Like(escape_like_starts_with(val)),
            StringCompare::EndsWith => StringCmpOp::Like(escape_like_ends_with(val)),
            StringCompare::Contains => StringCmpOp::Like(escape_like_contains(val)),
            StringCompare::Matches => StringCmpOp::Like(escape_like_matches(val)),
        };
        select = match string_cmp_op {
            StringCmpOp::Equal(eq) => {
                if dir {
                    select.filter(track_tag::label.eq(eq))
                } else {
                    select.filter(track_tag::label.ne(eq))
                }
            }
            StringCmpOp::Prefix(prefix, len) => {
                let sql_prefix_filter = if dir {
                    format!("substr(track_tag.label,1,{})='{}'", len, prefix)
                } else {
                    format!("substr(track_tag.label,1,{})<>'{}'", len, prefix)
                };
                select.filter(diesel::dsl::sql(&sql_prefix_filter))
            }
            StringCmpOp::Like(like) => {
                if dir {
                    select.filter(track_tag::label.like(like).escape(LIKE_ESCAPE_CHARACTER))
                } else {
                    select.filter(
                        track_tag::label
                            .not_like(like)
                            .escape(LIKE_ESCAPE_CHARACTER),
                    )
                }
            }
        };
    }

    // Filter tag score
    if let Some(score) = tag_filter.score {
        select = match score {
            NumericPredicate::LessThan(value) => select.filter(track_tag::score.lt(value)),
            NumericPredicate::GreaterOrEqual(value) => select.filter(track_tag::score.ge(value)),
            NumericPredicate::GreaterThan(value) => select.filter(track_tag::score.gt(value)),
            NumericPredicate::LessOrEqual(value) => select.filter(track_tag::score.le(value)),
            NumericPredicate::Equal(value) => {
                if let Some(value) = value {
                    select.filter(track_tag::score.eq(value))
                } else {
                    select.filter(track_tag::score.is_null())
                }
            }
            NumericPredicate::NotEqual(value) => {
                if let Some(value) = value {
                    select.filter(track_tag::score.ne(value))
                } else {
                    select.filter(track_tag::score.is_not_null())
                }
            }
        };
    }

    (select, tag_filter.modifier)
}

fn build_tag_filter_expression(filter: &TagFilter) -> TrackSearchBoxedExpression<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(&filter);
    match filter_modifier {
        None => Box::new(track::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(track::row_id.ne_all(subselect)),
    }
}

fn build_cue_label_filter_expression(
    filter: StringFilterBorrowed<'_>,
) -> TrackSearchBoxedExpression<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_cue_filter(filter);
    match filter_modifier {
        None => Box::new(track::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(track::row_id.ne_all(subselect)),
    }
}

fn select_track_ids_matching_cue_filter<'s, 'db, DB>(
    cue_label_filter: StringFilterBorrowed<'s>,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'db,
        diesel::sql_types::BigInt,
        track_cue::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'db,
{
    let mut select = track_cue::table.select(track_cue::track_id).into_boxed();

    // Filter labels
    if let Some(label) = cue_label_filter.value {
        let (cmp, val, dir) = label.into();
        let string_cmp_op = match cmp {
            // Equal comparison without escape characters
            StringCompare::Equals => StringCmpOp::Equal(val.to_owned()),
            StringCompare::Prefix => StringCmpOp::Prefix(escape_single_quotes(val), val.len()),
            // Like comparisons with escaped wildcard character
            StringCompare::StartsWith => StringCmpOp::Like(escape_like_starts_with(val)),
            StringCompare::EndsWith => StringCmpOp::Like(escape_like_ends_with(val)),
            StringCompare::Contains => StringCmpOp::Like(escape_like_contains(val)),
            StringCompare::Matches => StringCmpOp::Like(escape_like_matches(val)),
        };
        select = match string_cmp_op {
            StringCmpOp::Equal(eq) => {
                if dir {
                    select.filter(track_cue::label.eq(eq))
                } else {
                    select.filter(track_cue::label.ne(eq))
                }
            }
            StringCmpOp::Prefix(prefix, len) => {
                let sql_prefix_filter = if dir {
                    format!("substr(track_cue.label,1,{})='{}'", len, prefix)
                } else {
                    format!("substr(track_cue.label,1,{})<>'{}'", len, prefix)
                };
                select.filter(diesel::dsl::sql(&sql_prefix_filter))
            }
            StringCmpOp::Like(like) => {
                if dir {
                    select.filter(track_cue::label.like(like).escape(LIKE_ESCAPE_CHARACTER))
                } else {
                    select.filter(
                        track_cue::label
                            .not_like(like)
                            .escape(LIKE_ESCAPE_CHARACTER),
                    )
                }
            }
        };
    }

    (select, cue_label_filter.modifier)
}

impl TrackSearchBoxedExpressionBuilder for SearchFilter {
    fn build_expression(&self) -> TrackSearchBoxedExpression<'_> {
        use SearchFilter::*;
        match self {
            Phrase(filter) => build_phrase_field_filter_expression(filter),
            Numeric(filter) => build_numeric_field_filter_expression(filter),
            DateTime(filter) => build_datetime_field_filter_expression(filter),
            Condition(filter) => build_condition_filter_expression(*filter),
            Tag(filter) => build_tag_filter_expression(filter),
            CueLabel(filter) => build_cue_label_filter_expression(filter.borrow()),
            All(filters) => filters
                .iter()
                .fold(dummy_true_expression(), |expr, filter| {
                    Box::new(expr.and(filter.build_expression()))
                }),
            Any(filters) => filters
                .iter()
                .fold(dummy_false_expression(), |expr, filter| {
                    Box::new(expr.or(filter.build_expression()))
                }),
            Not(filter) => Box::new(diesel::dsl::not(filter.build_expression())),
        }
    }
}
