// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::{
    query_source::joins as diesel_joins,
    sql_types::{BigInt, Binary, Double, Integer, Nullable, SmallInt, Text},
    BoolExpressionMethods, BoxableExpression, ExpressionMethods, TextExpressionMethods,
};

use num_traits::ToPrimitive as _;

use aoide_core::{
    playlist::EntityUid as PlaylistUid, track::EntityUid as TrackUid, util::clock::YYYYMMDD,
};

use aoide_core_api::{tag::search::Filter as TagFilter, track::search::*};

use crate::{
    db::{
        media_source::schema::*, media_tracker::schema::*, playlist::schema::*,
        playlist_entry::schema::*, track::schema::*, track_actor::schema::*, track_cue::schema::*,
        track_tag::schema::*, track_title::schema::*,
    },
    prelude::*,
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
        Nullable<BigInt>,
        Nullable<Text>,
        Nullable<BigInt>,
        Nullable<Integer>,
        Nullable<Text>,
        Nullable<BigInt>,
        Nullable<Integer>,
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

pub(crate) trait TrackSearchBoxedExpressionBuilder {
    fn build_expression(&self) -> TrackSearchBoxedExpression<'_>;
}

pub(crate) trait TrackSearchQueryTransform {
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
            SortField::CollectedAt => match direction {
                SortDirection::Ascending => query.then_order_by(media_source::collected_ms.asc()),
                SortDirection::Descending => query.then_order_by(media_source::collected_ms.desc()),
            },
            SortField::ContentPath => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(media_source::content_link_path.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(media_source::content_link_path.desc())
                }
            },
            SortField::ContentType => match direction {
                SortDirection::Ascending => query.then_order_by(media_source::content_type.asc()),
                SortDirection::Descending => query.then_order_by(media_source::content_type.desc()),
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
            SortField::MusicTempoBpm => match direction {
                SortDirection::Ascending => query.then_order_by(track::music_tempo_bpm.asc()),
                SortDirection::Descending => query.then_order_by(track::music_tempo_bpm.desc()),
            },
            SortField::MusicKeyCode => match direction {
                SortDirection::Ascending => query.then_order_by(track::music_key_code.asc()),
                SortDirection::Descending => query.then_order_by(track::music_key_code.desc()),
            },
            SortField::Publisher => match direction {
                SortDirection::Ascending => query.then_order_by(track::publisher.asc()),
                SortDirection::Descending => query.then_order_by(track::publisher.desc()),
            },
            SortField::RecordedAtDate => match direction {
                SortDirection::Ascending => query.then_order_by(track::recorded_at_yyyymmdd.asc()),
                SortDirection::Descending => {
                    query.then_order_by(track::recorded_at_yyyymmdd.desc())
                }
            },
            SortField::ReleasedAtDate => match direction {
                SortDirection::Ascending => query.then_order_by(track::released_at_yyyymmdd.asc()),
                SortDirection::Descending => {
                    query.then_order_by(track::released_at_yyyymmdd.desc())
                }
            },
            SortField::ReleasedOrigAtDate => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(track::released_orig_at_yyyymmdd.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(track::released_orig_at_yyyymmdd.desc())
                }
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
            SortField::UpdatedAt => match direction {
                SortDirection::Ascending => query.then_order_by(track::row_updated_ms.asc()),
                SortDirection::Descending => query.then_order_by(track::row_updated_ms.desc()),
            },
        }
    }
}

fn build_track_uid_filter_expression(track_uid: &TrackUid) -> TrackSearchBoxedExpression<'_> {
    Box::new(track::entity_uid.eq(track_uid.as_ref()))
}

fn build_phrase_like_expr_escaped<'term>(
    terms: impl IntoIterator<Item = &'term str>,
) -> Option<String> {
    let escaped_terms: Vec<_> = terms.into_iter().map(escape_like_matches).collect();
    let escaped_terms_str_len = escaped_terms.iter().fold(0, |len, term| len + term.len());
    if escaped_terms_str_len == 0 {
        return None;
    }
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
    Some(like_expr)
}

fn build_phrase_field_filter_expression(
    filter: &PhraseFieldFilter,
) -> TrackSearchBoxedExpression<'_> {
    let like_expr = build_phrase_like_expr_escaped(filter.terms.iter().map(String::as_str));

    let mut or_expression = dummy_false_expression();
    // media_source (join)
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::ContentPath)
    {
        or_expression = if let Some(like_expr) = &like_expr {
            Box::new(
                or_expression.or(media_source::content_link_path
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(media_source::content_link_path.is_null())
                    .or(media_source::content_link_path.eq(String::default())),
            )
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::ContentType)
    {
        or_expression = if let Some(like_expr) = &like_expr {
            Box::new(
                or_expression.or(media_source::content_type
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(media_source::content_type.is_null())
                    .or(media_source::content_type.eq(String::default())),
            )
        };
    }
    // track (join)
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::Publisher)
    {
        or_expression = if let Some(like_expr) = like_expr {
            Box::new(
                or_expression.or(track::publisher
                    .like(like_expr)
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(track::publisher.is_null())
                    .or(track::publisher.eq(String::default())),
            )
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
        AdvisoryRating => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(media_source::advisory_rating.lt(value as i16)),
            LessOrEqual(value) => Box::new(media_source::advisory_rating.le(value as i16)),
            GreaterThan(value) => Box::new(media_source::advisory_rating.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(media_source::advisory_rating.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(media_source::advisory_rating.eq(value as i16))
                } else {
                    Box::new(media_source::advisory_rating.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(media_source::advisory_rating.ne(value as i16))
                } else {
                    Box::new(media_source::advisory_rating.is_not_null())
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
        RecordedAtDate => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to YYYYMMDD
            LessThan(value) => Box::new(track::recorded_at_yyyymmdd.lt(value as YYYYMMDD)),
            LessOrEqual(value) => Box::new(track::recorded_at_yyyymmdd.le(value as YYYYMMDD)),
            GreaterThan(value) => Box::new(track::recorded_at_yyyymmdd.gt(value as YYYYMMDD)),
            GreaterOrEqual(value) => Box::new(track::recorded_at_yyyymmdd.ge(value as YYYYMMDD)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::recorded_at_yyyymmdd.eq(value as YYYYMMDD))
                } else {
                    Box::new(track::recorded_at_yyyymmdd.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::recorded_at_yyyymmdd.ne(value as YYYYMMDD))
                } else {
                    Box::new(track::recorded_at_yyyymmdd.is_not_null())
                }
            }
        },
        ReleasedAtDate => match filter.predicate {
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
        ReleasedOrigAtDate => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to YYYYMMDD
            LessThan(value) => Box::new(track::released_orig_at_yyyymmdd.lt(value as YYYYMMDD)),
            LessOrEqual(value) => Box::new(track::released_orig_at_yyyymmdd.le(value as YYYYMMDD)),
            GreaterThan(value) => Box::new(track::released_orig_at_yyyymmdd.gt(value as YYYYMMDD)),
            GreaterOrEqual(value) => {
                Box::new(track::released_orig_at_yyyymmdd.ge(value as YYYYMMDD))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::released_orig_at_yyyymmdd.eq(value as YYYYMMDD))
                } else {
                    Box::new(track::released_orig_at_yyyymmdd.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::released_orig_at_yyyymmdd.ne(value as YYYYMMDD))
                } else {
                    Box::new(track::released_orig_at_yyyymmdd.is_not_null())
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
        CollectedAt => match filter.predicate {
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
        RecordedAt => match filter.predicate {
            LessThan(value) => Box::new(track::recorded_ms.lt(value.timestamp_millis())),
            LessOrEqual(value) => Box::new(track::recorded_ms.le(value.timestamp_millis())),
            GreaterThan(value) => Box::new(track::recorded_ms.gt(value.timestamp_millis())),
            GreaterOrEqual(value) => Box::new(track::recorded_ms.ge(value.timestamp_millis())),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::recorded_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(track::recorded_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::recorded_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(track::recorded_ms.is_not_null())
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
        ReleasedOrigAt => match filter.predicate {
            LessThan(value) => Box::new(track::released_orig_ms.lt(value.timestamp_millis())),
            LessOrEqual(value) => Box::new(track::released_orig_ms.le(value.timestamp_millis())),
            GreaterThan(value) => Box::new(track::released_orig_ms.gt(value.timestamp_millis())),
            GreaterOrEqual(value) => Box::new(track::released_orig_ms.ge(value.timestamp_millis())),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(track::released_orig_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(track::released_orig_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(track::released_orig_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(track::released_orig_ms.is_not_null())
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
    filter: &'a TagFilter,
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

    let TagFilter {
        modifier,
        facets,
        label,
        score,
    } = filter;

    // Filter facet(s)
    if let Some(ref facets) = facets {
        if facets.is_empty() {
            // unfaceted tags without a facet
            select = select.filter(track_tag::facet.is_null());
        } else {
            // tags with any of the given facets
            select = select.filter(track_tag::facet.eq_any(facets));
        }
    }

    // Filter labels
    if let Some(ref label) = label {
        let (val, cmp, dir) = decompose_string_predicate(label.borrow());
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
                    sql_column_substr_prefix_eq("track_tag.label", &prefix[..len])
                } else {
                    sql_column_substr_prefix_ne("track_tag.label", &prefix[..len])
                };
                select.filter(sql_prefix_filter)
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
    if let Some(score) = score {
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

    (select, *modifier)
}

fn build_tag_filter_expression(filter: &TagFilter) -> TrackSearchBoxedExpression<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(filter);
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
    filter: StringFilterBorrowed<'s>,
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
    if let Some(label) = filter.value {
        let (val, cmp, dir) = decompose_string_predicate(label);
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
                    sql_column_substr_prefix_eq("track_cue.label", &prefix[..len])
                } else {
                    sql_column_substr_prefix_ne("track_cue.label", &prefix[..len])
                };
                select.filter(sql_prefix_filter)
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

    (select, filter.modifier)
}

fn build_playlist_uid_filter_expression(
    playlist_uid: &PlaylistUid,
) -> TrackSearchBoxedExpression<'_> {
    let subselect = select_track_ids_matching_playlist_uid_filter(playlist_uid);
    Box::new(track::row_id.eq_any(subselect))
}

fn select_track_ids_matching_playlist_uid_filter<'db, DB>(
    playlist_uid: &'db PlaylistUid,
) -> diesel::query_builder::BoxedSelectStatement<'db, diesel::sql_types::BigInt, track::table, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    let subselect = playlist::table
        .inner_join(playlist_entry::table)
        .select(playlist_entry::track_id)
        .filter(playlist::entity_uid.eq(playlist_uid.as_ref()))
        .filter(playlist_entry::track_id.is_not_null());
    track::table
        .select(track::row_id)
        .filter(track::row_id.nullable().eq_any(subselect))
        .into_boxed()
}

fn select_track_ids_matching_actor_filter<'a, DB>(
    filter: &'a ActorPhraseFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        track_actor::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = track_actor::table
        .select(track_actor::track_id)
        .into_boxed();

    let ActorPhraseFilter {
        modifier,
        scope,
        roles,
        kinds,
        name_terms,
    } = filter;

    if let Some(scope) = scope {
        select = select.filter(track_actor::scope.eq(scope.to_i16().expect("actor scope")));
    }

    // Filter role(s)
    if !roles.is_empty() {
        select = select.filter(
            track_actor::role.eq_any(roles.iter().map(|role| role.to_i16().expect("actor role"))),
        );
    }

    // Filter kind(s)
    if !kinds.is_empty() {
        select = select.filter(
            track_actor::kind.eq_any(kinds.iter().map(|kind| kind.to_i16().expect("actor kind"))),
        );
    }

    // Filter name
    let name_like_expr_escaped =
        build_phrase_like_expr_escaped(name_terms.iter().map(String::as_str));
    if let Some(name_like_expr_escaped) = name_like_expr_escaped {
        select = select.filter(
            track_actor::name
                .like(name_like_expr_escaped)
                .escape(LIKE_ESCAPE_CHARACTER),
        );
    };

    (select, *modifier)
}

fn build_actor_filter_expression(filter: &ActorPhraseFilter) -> TrackSearchBoxedExpression<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_actor_filter(filter);
    match filter_modifier {
        None => Box::new(track::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(track::row_id.ne_all(subselect)),
    }
}

fn select_track_ids_matching_title_filter<'a, DB>(
    filter: &'a TitlePhraseFilter,
) -> (
    diesel::query_builder::BoxedSelectStatement<
        'a,
        diesel::sql_types::BigInt,
        track_title::table,
        DB,
    >,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'a,
{
    let mut select = track_title::table
        .select(track_title::track_id)
        .into_boxed();

    let TitlePhraseFilter {
        modifier,
        scope,
        kinds,
        name_terms,
    } = filter;

    if let Some(scope) = scope {
        select = select.filter(track_title::scope.eq(scope.to_i16().expect("title scope")));
    }

    // Filter kind(s)
    if !kinds.is_empty() {
        select = select.filter(
            track_title::kind.eq_any(kinds.iter().map(|kind| kind.to_i16().expect("title kind"))),
        );
    }

    // Filter name
    let name_like_expr_escaped =
        build_phrase_like_expr_escaped(name_terms.iter().map(String::as_str));
    if let Some(name_like_expr_escaped) = name_like_expr_escaped {
        select = select.filter(
            track_title::name
                .like(name_like_expr_escaped)
                .escape(LIKE_ESCAPE_CHARACTER),
        );
    };

    (select, *modifier)
}

fn build_title_filter_expression(filter: &TitlePhraseFilter) -> TrackSearchBoxedExpression<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_title_filter(filter);
    match filter_modifier {
        None => Box::new(track::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(track::row_id.ne_all(subselect)),
    }
}

impl TrackSearchBoxedExpressionBuilder for Filter {
    fn build_expression(&self) -> TrackSearchBoxedExpression<'_> {
        use Filter::*;
        match self {
            Phrase(filter) => build_phrase_field_filter_expression(filter),
            Numeric(filter) => build_numeric_field_filter_expression(filter),
            DateTime(filter) => build_datetime_field_filter_expression(filter),
            Condition(filter) => build_condition_filter_expression(*filter),
            Tag(filter) => build_tag_filter_expression(filter),
            CueLabel(filter) => build_cue_label_filter_expression(filter.borrow()),
            TrackUid(track_uid) => build_track_uid_filter_expression(track_uid),
            PlaylistUid(playlist_uid) => build_playlist_uid_filter_expression(playlist_uid),
            ActorPhrase(filter) => build_actor_filter_expression(filter),
            TitlePhrase(filter) => build_title_filter_expression(filter),
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

/// (Value, Comparison, Include(true)/Exclude(false))
fn decompose_string_predicate(p: StringPredicateBorrowed<'_>) -> (&str, StringCompare, bool) {
    use StringPredicateBorrowed::*;
    match p {
        StartsWith(s) => (s, StringCompare::StartsWith, true),
        StartsNotWith(s) => (s, StringCompare::StartsWith, false),
        EndsWith(s) => (s, StringCompare::EndsWith, true),
        EndsNotWith(s) => (s, StringCompare::EndsWith, false),
        Contains(s) => (s, StringCompare::Contains, true),
        ContainsNot(s) => (s, StringCompare::Contains, false),
        Matches(s) => (s, StringCompare::Matches, true),
        MatchesNot(s) => (s, StringCompare::Matches, false),
        Prefix(s) => (s, StringCompare::Prefix, true),
        Equals(s) => (s, StringCompare::Equals, true),
        EqualsNot(s) => (s, StringCompare::Equals, false),
    }
}
