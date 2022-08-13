// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::{BoolExpressionMethods, BoxableExpression, ExpressionMethods, TextExpressionMethods};

use num_traits::ToPrimitive as _;

use aoide_core::{
    playlist::EntityUid as PlaylistUid, track::EntityUid as TrackUid, util::clock::YYYYMMDD,
};

use aoide_core_api::{tag::search::Filter as TagFilter, track::search::*};

use crate::{
    db::{
        media_tracker::schema::*, playlist::schema::*, playlist_entry::schema::*,
        track_actor::schema::*, track_cue::schema::*, track_tag::schema::*, track_title::schema::*,
        view_track_search::schema::*,
    },
    prelude::*,
};

///////////////////////////////////////////////////////////////////////

type TrackSearchBoxedExpression<'db, DB> = Box<
    dyn BoxableExpression<view_track_search::table, DB, SqlType = diesel::sql_types::Bool> + 'db,
>;

// TODO: replace with "True"
fn dummy_true_expression<'db, DB>() -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    Box::new(view_track_search::row_id.is_not_null()) // always true
}

// TODO: replace with "False"
fn dummy_false_expression<'db, DB>() -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    Box::new(view_track_search::row_id.is_null()) // always false
}

pub(crate) trait TrackSearchBoxedExpressionBuilder<'db, DB> {
    fn build_expression(&'db self) -> TrackSearchBoxedExpression<'db, DB>;
}

pub(crate) trait TrackSearchQueryTransform<'db, DB> {
    fn apply_to_query(
        &'db self,
        query: view_track_search::BoxedQuery<'db, DB>,
    ) -> view_track_search::BoxedQuery<'db, DB>;
}

impl<'db, DB> TrackSearchQueryTransform<'db, DB> for SortOrder
where
    DB: diesel::backend::Backend + 'db,
{
    fn apply_to_query(
        &self,
        query: view_track_search::BoxedQuery<'db, DB>,
    ) -> view_track_search::BoxedQuery<'db, DB> {
        let direction = self.direction;
        match self.field {
            SortField::AudioBitrateBps => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::audio_bitrate_bps.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::audio_bitrate_bps.desc())
                }
            },
            SortField::AudioChannelCount => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::audio_channel_count.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::audio_channel_count.desc())
                }
            },
            SortField::AudioDurationMs => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::audio_duration_ms.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::audio_duration_ms.desc())
                }
            },
            SortField::AudioLoudnessLufs => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::audio_loudness_lufs.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::audio_loudness_lufs.desc())
                }
            },
            SortField::AudioSampleRateHz => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::audio_samplerate_hz.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::audio_samplerate_hz.desc())
                }
            },
            SortField::CollectedAt => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::collected_ms.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::collected_ms.desc())
                }
            },
            SortField::ContentPath => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::content_link_path.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::content_link_path.desc())
                }
            },
            SortField::ContentType => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::content_type.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::content_type.desc())
                }
            },
            SortField::Copyright => match direction {
                SortDirection::Ascending => query.then_order_by(view_track_search::copyright.asc()),
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::copyright.desc())
                }
            },
            SortField::CreatedAt => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::row_created_ms.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::row_created_ms.desc())
                }
            },
            SortField::DiscNumber => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::disc_number.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::disc_number.desc())
                }
            },
            SortField::DiscTotal => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::disc_total.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::disc_total.desc())
                }
            },
            SortField::MusicTempoBpm => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::music_tempo_bpm.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::music_tempo_bpm.desc())
                }
            },
            SortField::MusicKeyCode => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::music_key_code.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::music_key_code.desc())
                }
            },
            SortField::Publisher => match direction {
                SortDirection::Ascending => query.then_order_by(view_track_search::publisher.asc()),
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::publisher.desc())
                }
            },
            SortField::RecordedAtDate => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::recorded_at_yyyymmdd.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::recorded_at_yyyymmdd.desc())
                }
            },
            SortField::ReleasedAtDate => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::released_at_yyyymmdd.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::released_at_yyyymmdd.desc())
                }
            },
            SortField::ReleasedOrigAtDate => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::released_orig_at_yyyymmdd.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::released_orig_at_yyyymmdd.desc())
                }
            },
            SortField::TrackNumber => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::track_number.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::track_number.desc())
                }
            },
            SortField::TrackTotal => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::track_total.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::track_total.desc())
                }
            },
            SortField::UpdatedAt => match direction {
                SortDirection::Ascending => {
                    query.then_order_by(view_track_search::row_updated_ms.asc())
                }
                SortDirection::Descending => {
                    query.then_order_by(view_track_search::row_updated_ms.desc())
                }
            },
        }
    }
}

fn build_any_track_uid_filter_expression<'db, DB>(
    any_track_uid: &'db [TrackUid],
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    Box::new(view_track_search::entity_uid.eq_any(any_track_uid.iter().map(|uid| uid.as_ref())))
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

fn build_phrase_field_filter_expression<'db, DB>(
    filter: &PhraseFieldFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
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
                or_expression.or(view_track_search::content_link_path
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(view_track_search::content_link_path.is_null())
                    .or(view_track_search::content_link_path.eq(String::default())),
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
                or_expression.or(view_track_search::content_type
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(view_track_search::content_type.is_null())
                    .or(view_track_search::content_type.eq(String::default())),
            )
        };
    }
    // track (join)
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::Copyright)
    {
        or_expression = if let Some(like_expr) = &like_expr {
            Box::new(
                or_expression.or(view_track_search::copyright
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(view_track_search::copyright.is_null())
                    .or(view_track_search::copyright.eq(String::default())),
            )
        };
    }
    if filter.fields.is_empty()
        || filter
            .fields
            .iter()
            .any(|target| *target == StringField::Publisher)
    {
        or_expression = if let Some(like_expr) = like_expr {
            Box::new(
                or_expression.or(view_track_search::publisher
                    .like(like_expr)
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(
                or_expression
                    .or(view_track_search::publisher.is_null())
                    .or(view_track_search::publisher.eq(String::default())),
            )
        };
    }
    or_expression
}

fn build_numeric_field_filter_expression<'db, DB>(
    filter: &NumericFieldFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    use NumericField::*;
    use ScalarPredicate::*;
    match filter.field {
        AudioDurationMs => match filter.predicate {
            LessThan(value) => Box::new(view_track_search::audio_duration_ms.lt(value)),
            LessOrEqual(value) => Box::new(view_track_search::audio_duration_ms.le(value)),
            GreaterThan(value) => Box::new(view_track_search::audio_duration_ms.gt(value)),
            GreaterOrEqual(value) => Box::new(view_track_search::audio_duration_ms.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_duration_ms.eq(value))
                } else {
                    Box::new(view_track_search::audio_duration_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_duration_ms.ne(value))
                } else {
                    Box::new(view_track_search::audio_duration_ms.is_not_null())
                }
            }
        },
        AudioSampleRateHz => match filter.predicate {
            LessThan(value) => Box::new(view_track_search::audio_samplerate_hz.lt(value)),
            LessOrEqual(value) => Box::new(view_track_search::audio_samplerate_hz.le(value)),
            GreaterThan(value) => Box::new(view_track_search::audio_samplerate_hz.gt(value)),
            GreaterOrEqual(value) => Box::new(view_track_search::audio_samplerate_hz.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_samplerate_hz.eq(value))
                } else {
                    Box::new(view_track_search::audio_samplerate_hz.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_samplerate_hz.ne(value))
                } else {
                    Box::new(view_track_search::audio_samplerate_hz.is_not_null())
                }
            }
        },
        AudioBitrateBps => match filter.predicate {
            LessThan(value) => Box::new(view_track_search::audio_bitrate_bps.lt(value)),
            LessOrEqual(value) => Box::new(view_track_search::audio_bitrate_bps.le(value)),
            GreaterThan(value) => Box::new(view_track_search::audio_bitrate_bps.gt(value)),
            GreaterOrEqual(value) => Box::new(view_track_search::audio_bitrate_bps.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_bitrate_bps.eq(value))
                } else {
                    Box::new(view_track_search::audio_bitrate_bps.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_bitrate_bps.ne(value))
                } else {
                    Box::new(view_track_search::audio_bitrate_bps.is_not_null())
                }
            }
        },
        AudioChannelCount => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::audio_channel_count.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::audio_channel_count.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::audio_channel_count.gt(value as i16)),
            GreaterOrEqual(value) => {
                Box::new(view_track_search::audio_channel_count.ge(value as i16))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_channel_count.eq(value as i16))
                } else {
                    Box::new(view_track_search::audio_channel_count.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_channel_count.ne(value as i16))
                } else {
                    Box::new(view_track_search::audio_channel_count.is_not_null())
                }
            }
        },
        AudioLoudnessLufs => match filter.predicate {
            LessThan(value) => Box::new(view_track_search::audio_loudness_lufs.lt(value)),
            LessOrEqual(value) => Box::new(view_track_search::audio_loudness_lufs.le(value)),
            GreaterThan(value) => Box::new(view_track_search::audio_loudness_lufs.gt(value)),
            GreaterOrEqual(value) => Box::new(view_track_search::audio_loudness_lufs.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_loudness_lufs.eq(value))
                } else {
                    Box::new(view_track_search::audio_loudness_lufs.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::audio_loudness_lufs.ne(value))
                } else {
                    Box::new(view_track_search::audio_loudness_lufs.is_not_null())
                }
            }
        },
        AdvisoryRating => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::advisory_rating.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::advisory_rating.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::advisory_rating.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(view_track_search::advisory_rating.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::advisory_rating.eq(value as i16))
                } else {
                    Box::new(view_track_search::advisory_rating.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::advisory_rating.ne(value as i16))
                } else {
                    Box::new(view_track_search::advisory_rating.is_not_null())
                }
            }
        },
        TrackNumber => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::track_number.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::track_number.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::track_number.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(view_track_search::track_number.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::track_number.eq(value as i16))
                } else {
                    Box::new(view_track_search::track_number.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::track_number.ne(value as i16))
                } else {
                    Box::new(view_track_search::track_number.is_not_null())
                }
            }
        },
        TrackTotal => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::track_total.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::track_total.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::track_total.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(view_track_search::track_total.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::track_total.eq(value as i16))
                } else {
                    Box::new(view_track_search::track_total.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::track_total.ne(value as i16))
                } else {
                    Box::new(view_track_search::track_total.is_not_null())
                }
            }
        },
        DiscNumber => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::disc_number.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::disc_number.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::disc_number.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(view_track_search::disc_number.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::disc_number.eq(value as i16))
                } else {
                    Box::new(view_track_search::disc_number.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::disc_number.ne(value as i16))
                } else {
                    Box::new(view_track_search::disc_number.is_not_null())
                }
            }
        },
        DiscTotal => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::disc_total.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::disc_total.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::disc_total.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(view_track_search::disc_total.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::disc_total.eq(value as i16))
                } else {
                    Box::new(view_track_search::disc_total.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::disc_total.ne(value as i16))
                } else {
                    Box::new(view_track_search::disc_total.is_not_null())
                }
            }
        },
        RecordedAtDate => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to YYYYMMDD
            LessThan(value) => {
                Box::new(view_track_search::recorded_at_yyyymmdd.lt(value as YYYYMMDD))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::recorded_at_yyyymmdd.le(value as YYYYMMDD))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::recorded_at_yyyymmdd.gt(value as YYYYMMDD))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::recorded_at_yyyymmdd.ge(value as YYYYMMDD))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::recorded_at_yyyymmdd.eq(value as YYYYMMDD))
                } else {
                    Box::new(view_track_search::recorded_at_yyyymmdd.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::recorded_at_yyyymmdd.ne(value as YYYYMMDD))
                } else {
                    Box::new(view_track_search::recorded_at_yyyymmdd.is_not_null())
                }
            }
        },
        ReleasedAtDate => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to YYYYMMDD
            LessThan(value) => {
                Box::new(view_track_search::released_at_yyyymmdd.lt(value as YYYYMMDD))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::released_at_yyyymmdd.le(value as YYYYMMDD))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::released_at_yyyymmdd.gt(value as YYYYMMDD))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::released_at_yyyymmdd.ge(value as YYYYMMDD))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_at_yyyymmdd.eq(value as YYYYMMDD))
                } else {
                    Box::new(view_track_search::released_at_yyyymmdd.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_at_yyyymmdd.ne(value as YYYYMMDD))
                } else {
                    Box::new(view_track_search::released_at_yyyymmdd.is_not_null())
                }
            }
        },
        ReleasedOrigAtDate => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to YYYYMMDD
            LessThan(value) => {
                Box::new(view_track_search::released_orig_at_yyyymmdd.lt(value as YYYYMMDD))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::released_orig_at_yyyymmdd.le(value as YYYYMMDD))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::released_orig_at_yyyymmdd.gt(value as YYYYMMDD))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::released_orig_at_yyyymmdd.ge(value as YYYYMMDD))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_orig_at_yyyymmdd.eq(value as YYYYMMDD))
                } else {
                    Box::new(view_track_search::released_orig_at_yyyymmdd.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_orig_at_yyyymmdd.ne(value as YYYYMMDD))
                } else {
                    Box::new(view_track_search::released_orig_at_yyyymmdd.is_not_null())
                }
            }
        },
        MusicTempoBpm => match filter.predicate {
            LessThan(value) => Box::new(view_track_search::music_tempo_bpm.lt(value)),
            LessOrEqual(value) => Box::new(view_track_search::music_tempo_bpm.le(value)),
            GreaterThan(value) => Box::new(view_track_search::music_tempo_bpm.gt(value)),
            GreaterOrEqual(value) => Box::new(view_track_search::music_tempo_bpm.ge(value)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::music_tempo_bpm.eq(value))
                } else {
                    Box::new(view_track_search::music_tempo_bpm.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::music_tempo_bpm.ne(value))
                } else {
                    Box::new(view_track_search::music_tempo_bpm.is_not_null())
                }
            }
        },
        MusicKeyCode => match filter.predicate {
            // TODO: Check and limit/clamp value range when converting from f64 to i16
            LessThan(value) => Box::new(view_track_search::music_key_code.lt(value as i16)),
            LessOrEqual(value) => Box::new(view_track_search::music_key_code.le(value as i16)),
            GreaterThan(value) => Box::new(view_track_search::music_key_code.gt(value as i16)),
            GreaterOrEqual(value) => Box::new(view_track_search::music_key_code.ge(value as i16)),
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::music_key_code.eq(value as i16))
                } else {
                    Box::new(view_track_search::music_key_code.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::music_key_code.ne(value as i16))
                } else {
                    Box::new(view_track_search::music_key_code.is_not_null())
                }
            }
        },
    }
}

fn build_datetime_field_filter_expression<'db, DB>(
    filter: &DateTimeFieldFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    use DateTimeField::*;
    use ScalarPredicate::*;
    match filter.field {
        CollectedAt => match filter.predicate {
            LessThan(value) => {
                Box::new(view_track_search::collected_ms.lt(value.timestamp_millis()))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::collected_ms.le(value.timestamp_millis()))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::collected_ms.gt(value.timestamp_millis()))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::collected_ms.ge(value.timestamp_millis()))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::collected_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::collected_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::collected_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::collected_ms.is_not_null())
                }
            }
        },
        RecordedAt => match filter.predicate {
            LessThan(value) => {
                Box::new(view_track_search::recorded_ms.lt(value.timestamp_millis()))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::recorded_ms.le(value.timestamp_millis()))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::recorded_ms.gt(value.timestamp_millis()))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::recorded_ms.ge(value.timestamp_millis()))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::recorded_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::recorded_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::recorded_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::recorded_ms.is_not_null())
                }
            }
        },
        ReleasedAt => match filter.predicate {
            LessThan(value) => {
                Box::new(view_track_search::released_ms.lt(value.timestamp_millis()))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::released_ms.le(value.timestamp_millis()))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::released_ms.gt(value.timestamp_millis()))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::released_ms.ge(value.timestamp_millis()))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::released_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::released_ms.is_not_null())
                }
            }
        },
        ReleasedOrigAt => match filter.predicate {
            LessThan(value) => {
                Box::new(view_track_search::released_orig_ms.lt(value.timestamp_millis()))
            }
            LessOrEqual(value) => {
                Box::new(view_track_search::released_orig_ms.le(value.timestamp_millis()))
            }
            GreaterThan(value) => {
                Box::new(view_track_search::released_orig_ms.gt(value.timestamp_millis()))
            }
            GreaterOrEqual(value) => {
                Box::new(view_track_search::released_orig_ms.ge(value.timestamp_millis()))
            }
            Equal(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_orig_ms.eq(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::released_orig_ms.is_null())
                }
            }
            NotEqual(value) => {
                if let Some(value) = value {
                    Box::new(view_track_search::released_orig_ms.ne(value.timestamp_millis()))
                } else {
                    Box::new(view_track_search::released_orig_ms.is_not_null())
                }
            }
        },
    }
}

fn build_condition_filter_expression<'db, DB>(
    filter: ConditionFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    use ConditionFilter::*;
    match filter {
        SourceTracked => Box::new(
            view_track_search::media_source_id
                .eq_any(media_tracker_source::table.select(media_tracker_source::source_id)),
        ),
        SourceUntracked => Box::new(
            view_track_search::media_source_id
                .ne_all(media_tracker_source::table.select(media_tracker_source::source_id)),
        ),
    }
}

fn select_track_ids_matching_tag_filter<'db, DB>(
    filter: &'db TagFilter,
) -> (
    track_tag::BoxedQuery<'db, DB, diesel::sql_types::BigInt>,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'db,
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

fn build_tag_filter_expression<'db, DB>(
    filter: &'db TagFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

fn build_cue_label_filter_expression<'db, DB>(
    filter: StringFilterBorrowed<'_>,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    let (subselect, filter_modifier) = select_track_ids_matching_cue_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

fn select_track_ids_matching_cue_filter<'s, 'db, DB>(
    filter: StringFilterBorrowed<'s>,
) -> (
    track_cue::BoxedQuery<'db, DB, diesel::sql_types::BigInt>,
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

fn build_any_playlist_uid_filter_expression<'db, DB>(
    any_playlist_uid: &'db [PlaylistUid],
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    let subselect = select_track_ids_matching_any_playlist_uid_filter(any_playlist_uid);
    Box::new(view_track_search::row_id.eq_any(subselect))
}

fn select_track_ids_matching_any_playlist_uid_filter<'db, DB>(
    any_playlist_uid: impl IntoIterator<Item = &'db PlaylistUid>,
) -> view_track_search::BoxedQuery<'db, DB, diesel::sql_types::BigInt>
where
    DB: diesel::backend::Backend + 'db,
{
    let subselect = playlist::table
        .inner_join(playlist_entry::table)
        .select(playlist_entry::track_id)
        .filter(playlist::entity_uid.eq_any(any_playlist_uid.into_iter().map(|uid| uid.as_ref())))
        .filter(playlist_entry::track_id.is_not_null());
    view_track_search::table
        .select(view_track_search::row_id)
        .filter(view_track_search::row_id.nullable().eq_any(subselect))
        .into_boxed()
}

fn select_track_ids_matching_actor_filter<'db, DB>(
    filter: &'db ActorPhraseFilter,
) -> (
    track_actor::BoxedQuery<'db, DB, diesel::sql_types::BigInt>,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'db,
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

fn build_actor_filter_expression<'db, DB>(
    filter: &'db ActorPhraseFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    let (subselect, filter_modifier) = select_track_ids_matching_actor_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

fn select_track_ids_matching_title_filter<'db, DB>(
    filter: &'db TitlePhraseFilter,
) -> (
    track_title::BoxedQuery<'db, DB, diesel::sql_types::BigInt>,
    Option<FilterModifier>,
)
where
    DB: diesel::backend::Backend + 'db,
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

fn build_title_filter_expression<'db, DB>(
    filter: &'db TitlePhraseFilter,
) -> TrackSearchBoxedExpression<'db, DB>
where
    DB: diesel::backend::Backend + 'db,
{
    let (subselect, filter_modifier) = select_track_ids_matching_title_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

impl<'db, DB> TrackSearchBoxedExpressionBuilder<'db, DB> for Filter
where
    DB: diesel::backend::Backend + 'db,
{
    fn build_expression(&'db self) -> TrackSearchBoxedExpression<'db, DB> {
        use Filter::*;
        match self {
            Phrase(filter) => build_phrase_field_filter_expression(filter),
            Numeric(filter) => build_numeric_field_filter_expression(filter),
            DateTime(filter) => build_datetime_field_filter_expression(filter),
            Condition(filter) => build_condition_filter_expression(*filter),
            Tag(filter) => build_tag_filter_expression(filter),
            CueLabel(filter) => build_cue_label_filter_expression(filter.borrow()),
            AnyTrackUid(any_track_uid) => build_any_track_uid_filter_expression(any_track_uid),
            AnyPlaylistUid(any_playlist_uid) => {
                build_any_playlist_uid_filter_expression(any_playlist_uid)
            }
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
