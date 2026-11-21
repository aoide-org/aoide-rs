// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![allow(
    clippy::enum_glob_use,
    reason = "Improve readability when matching fields and predicates."
)]

use diesel::{prelude::*, sql_types};

use aoide_core::{
    PlaylistUid, TrackUid,
    audio::{
        ChannelFlags, DurationMs,
        channel::ChannelCount,
        signal::{BitrateBps, LoudnessLufs, SampleRateHz},
    },
    tag::FacetKey,
    util::clock::YyyyMmDdDateValue,
};
use aoide_core_api::{
    SortDirection,
    filter::{FilterModifier, NumericPredicate, ScalarPredicate, StringFilter},
    tag::search::{FacetsFilter, Filter as TagFilter},
    track::search::{
        ActorPhraseFilter, ConditionFilter, DateTimeField, DateTimeFieldFilter,
        Filter as TrackFilter, NumericField, NumericFieldFilter, PhraseFieldFilter, SortField,
        SortOrder, StringField, TitlePhraseFilter,
    },
};

use crate::{
    DbBackend,
    db::{
        media_tracker::schema::*,
        playlist::schema::*,
        playlist_entry::schema::*,
        track::{encode_advisory_rating, encode_search_scope},
        track_actor::schema::*,
        track_cue::schema::*,
        track_tag::schema::*,
        track_title::schema::*,
        view_track_search::schema::*,
    },
    util::{
        LIKE_ESCAPE_CHARACTER, LIKE_WILDCARD_CHARACTER, StringCmpOp, decompose_string_predicate,
        entity::encode_entity_uid, escape_like_matches, sql_column_substr_prefix_eq,
        sql_column_substr_prefix_ne,
    },
};

define_sql_function! { fn ifnull<ST: sql_types::SingleValue>(x: sql_types::Nullable<ST>, y: ST) -> ST; }

type TrackSearchExpressionBoxed<'db> = Box<
    dyn BoxableExpression<view_track_search::table, DbBackend, SqlType = sql_types::Bool> + 'db,
>;

// TODO: replace with "True"
fn dummy_true_expression() -> TrackSearchExpressionBoxed<'static> {
    Box::new(view_track_search::row_id.is_not_null()) // always true
}

// TODO: replace with "False"
fn dummy_false_expression() -> TrackSearchExpressionBoxed<'static> {
    Box::new(view_track_search::row_id.is_null()) // always false
}

#[expect(clippy::too_many_lines)] // TODO
pub(crate) fn apply_sort_order(
    sort_order: SortOrder,
    query: view_track_search::BoxedQuery<'_, DbBackend>,
) -> view_track_search::BoxedQuery<'_, DbBackend> {
    let direction = sort_order.direction;
    match sort_order.field {
        SortField::AlbumArtist => match direction {
            SortDirection::Ascending => {
                query.then_order_by(view_track_search::sort_album_artist.asc())
            }
            SortDirection::Descending => {
                query.then_order_by(view_track_search::sort_album_artist.desc())
            }
        },
        SortField::AlbumTitle => match direction {
            SortDirection::Ascending => {
                query.then_order_by(view_track_search::sort_album_title.asc())
            }
            SortDirection::Descending => {
                query.then_order_by(view_track_search::sort_album_title.desc())
            }
        },
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
        SortField::AudioChannelMask => match direction {
            SortDirection::Ascending => {
                query.then_order_by(view_track_search::audio_channel_mask.asc())
            }
            SortDirection::Descending => {
                query.then_order_by(view_track_search::audio_channel_mask.desc())
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
            SortDirection::Ascending => query.then_order_by(view_track_search::collected_ms.asc()),
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
            SortDirection::Ascending => query.then_order_by(view_track_search::content_type.asc()),
            SortDirection::Descending => {
                query.then_order_by(view_track_search::content_type.desc())
            }
        },
        SortField::Copyright => match direction {
            SortDirection::Ascending => query.then_order_by(view_track_search::copyright.asc()),
            SortDirection::Descending => query.then_order_by(view_track_search::copyright.desc()),
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
            SortDirection::Ascending => query.then_order_by(view_track_search::disc_number.asc()),
            SortDirection::Descending => query.then_order_by(view_track_search::disc_number.desc()),
        },
        SortField::DiscTotal => match direction {
            SortDirection::Ascending => query.then_order_by(view_track_search::disc_total.asc()),
            SortDirection::Descending => query.then_order_by(view_track_search::disc_total.desc()),
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
            SortDirection::Descending => query.then_order_by(view_track_search::publisher.desc()),
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
        SortField::TrackArtist => match direction {
            SortDirection::Ascending => {
                query.then_order_by(view_track_search::sort_track_artist.asc())
            }
            SortDirection::Descending => {
                query.then_order_by(view_track_search::sort_track_artist.desc())
            }
        },
        SortField::TrackNumber => match direction {
            SortDirection::Ascending => query.then_order_by(view_track_search::track_number.asc()),
            SortDirection::Descending => {
                query.then_order_by(view_track_search::track_number.desc())
            }
        },
        SortField::TrackTitle => match direction {
            SortDirection::Ascending => {
                query.then_order_by(view_track_search::sort_track_title.asc())
            }
            SortDirection::Descending => {
                query.then_order_by(view_track_search::sort_track_title.desc())
            }
        },
        SortField::TrackTotal => match direction {
            SortDirection::Ascending => query.then_order_by(view_track_search::track_total.asc()),
            SortDirection::Descending => query.then_order_by(view_track_search::track_total.desc()),
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

fn build_any_track_uid_filter_expression(
    any_track_uid: &[TrackUid],
) -> TrackSearchExpressionBoxed<'_> {
    Box::new(
        view_track_search::entity_uid.eq_any(
            any_track_uid
                .iter()
                .map(|track_uid| encode_entity_uid(track_uid).to_string()),
        ),
    )
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
) -> TrackSearchExpressionBoxed<'_> {
    let like_expr = build_phrase_like_expr_escaped(filter.terms.iter().map(String::as_str));

    let mut or_expression = dummy_false_expression();
    // media_source (join)
    if filter.fields.is_empty() || filter.fields.contains(&StringField::ContentPath) {
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
    if filter.fields.is_empty() || filter.fields.contains(&StringField::ContentType) {
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
    if filter.fields.is_empty() || filter.fields.contains(&StringField::Copyright) {
        let copyright_not_null = ifnull(view_track_search::copyright, "");
        or_expression = if let Some(like_expr) = &like_expr {
            Box::new(
                or_expression.or(copyright_not_null
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(or_expression.or(copyright_not_null.eq("")))
        };
    }
    if filter.fields.is_empty() || filter.fields.contains(&StringField::Publisher) {
        let publisher_not_null = ifnull(view_track_search::publisher, "");
        or_expression = if let Some(like_expr) = &like_expr {
            Box::new(
                or_expression.or(publisher_not_null
                    .like(like_expr.clone())
                    .escape(LIKE_ESCAPE_CHARACTER)),
            )
        } else {
            Box::new(or_expression.or(publisher_not_null.eq("")))
        };
    }
    or_expression
}

/// Builds a boxed filter expression from a [`ScalarPredicate`].
///
/// - `$predicate`: the predicate to match on
/// - `$expr`: the nullable column, used for `IS [NOT] NULL` comparisons
/// - `$expr_lt_le_eq_ne`: the expression for `<`, `<=`, `=`, `<>` comparisons
/// - `$expr_gt_ge`: the expression for `>` and `>=` comparisons (defaults to `$expr_lt_le_eq_ne`)
/// - `|$value| $convert`: converts the predicate value into the SQL value
macro_rules! scalar_predicate_expression {
    ($predicate:expr, $expr:expr, $expr_not_null:expr, |$value:ident| $convert:expr) => {
        scalar_predicate_expression!(
            $predicate,
            $expr,
            $expr_not_null,
            $expr_not_null,
            |$value| $convert
        )
    };
    (
        $predicate:expr,
        $expr:expr,
        $expr_lt_le_eq_ne:expr,
        $expr_gt_ge:expr,
        |$value:ident| $convert:expr
    ) => {
        match $predicate {
            ScalarPredicate::LessThan($value) => Box::new($expr_lt_le_eq_ne.lt($convert)),
            ScalarPredicate::LessOrEqual($value) => Box::new($expr_lt_le_eq_ne.le($convert)),
            ScalarPredicate::GreaterThan($value) => Box::new($expr_gt_ge.gt($convert)),
            ScalarPredicate::GreaterOrEqual($value) => Box::new($expr_gt_ge.ge($convert)),
            ScalarPredicate::Equal(Some($value)) => Box::new($expr_lt_le_eq_ne.eq($convert)),
            ScalarPredicate::Equal(None) => Box::new($expr.is_null()),
            ScalarPredicate::NotEqual(Some($value)) => Box::new($expr_lt_le_eq_ne.ne($convert)),
            ScalarPredicate::NotEqual(None) => Box::new($expr.is_not_null()),
        }
    };
}

// TODO: Check and limit/clamp value range when converting from f64 to integer types
fn build_numeric_field_filter_expression(
    filter: &NumericFieldFilter,
) -> TrackSearchExpressionBoxed<'_> {
    use NumericField::*;
    let predicate = &filter.predicate;
    match filter.field {
        AdvisoryRating => {
            let expr = view_track_search::advisory_rating;
            let expr_not_null = ifnull(expr, encode_advisory_rating(Default::default()));
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v as i16)
        }
        ArtworkDataSize => {
            let expr = view_track_search::artwork_data_size;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i64)
        }
        ArtworkImageHeight => {
            let expr = view_track_search::artwork_image_height;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i16)
        }
        ArtworkImageWidth => {
            let expr = view_track_search::artwork_image_width;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i16)
        }
        AudioDurationMs => {
            let expr = view_track_search::audio_duration_ms;
            let expr_not_null = ifnull(expr, DurationMs::empty().value());
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v)
        }
        AudioSampleRateHz => {
            let expr = view_track_search::audio_samplerate_hz;
            let expr_not_null = ifnull(expr, SampleRateHz::default().value());
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v)
        }
        AudioBitrateBps => {
            let expr = view_track_search::audio_bitrate_bps;
            let expr_not_null = ifnull(expr, BitrateBps::default().value());
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v)
        }
        AudioChannelCount => {
            let expr = view_track_search::audio_channel_count;
            let expr_not_null = ifnull(expr, ChannelCount::default().value() as i16);
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v as i16)
        }
        AudioChannelMask => {
            let expr = view_track_search::audio_channel_mask;
            let expr_not_null = ifnull(expr, ChannelFlags::default().bits() as i32);
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v as i32)
        }
        AudioLoudnessLufs => {
            let expr = view_track_search::audio_loudness_lufs;
            let expr_not_null = ifnull(expr, LoudnessLufs::default().value());
            scalar_predicate_expression!(predicate, expr, expr_not_null, |v| *v)
        }
        TrackNumber => {
            let expr = view_track_search::track_number;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i16)
        }
        TrackTotal => {
            let expr = view_track_search::track_total;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i16)
        }
        DiscNumber => {
            let expr = view_track_search::disc_number;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i16)
        }
        DiscTotal => {
            let expr = view_track_search::disc_total;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v as i16)
        }
        RecordedAtDate => {
            let expr = view_track_search::recorded_at_yyyymmdd;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v
                as YyyyMmDdDateValue)
        }
        ReleasedAtDate => {
            let expr = view_track_search::released_at_yyyymmdd;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v
                as YyyyMmDdDateValue)
        }
        ReleasedOrigAtDate => {
            let expr = view_track_search::released_orig_at_yyyymmdd;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0), |v| *v
                as YyyyMmDdDateValue)
        }
        MusicTempoBpm => {
            let expr = view_track_search::music_tempo_bpm;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0.0), |v| *v)
        }
        MusicKeyCode => {
            let expr = view_track_search::music_key_code;
            let expr_not_null_less_or_equal = ifnull(expr, i16::MAX);
            let expr_not_null_greater = ifnull(expr, -1);
            scalar_predicate_expression!(
                predicate,
                expr,
                expr_not_null_less_or_equal,
                expr_not_null_greater,
                |v| *v as i16
            )
        }
    }
}

fn build_datetime_field_filter_expression(
    filter: &DateTimeFieldFilter,
) -> TrackSearchExpressionBoxed<'_> {
    use DateTimeField::*;
    let predicate = &filter.predicate;
    match filter.field {
        CollectedAt => {
            let expr = view_track_search::collected_ms;
            scalar_predicate_expression!(predicate, expr, expr, |v| v.unix_timestamp_millis())
        }
        RecordedAt => {
            let expr = view_track_search::recorded_ms;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0i64), |v| v
                .unix_timestamp_millis())
        }
        ReleasedAt => {
            let expr = view_track_search::released_ms;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0i64), |v| v
                .unix_timestamp_millis())
        }
        ReleasedOrigAt => {
            let expr = view_track_search::released_orig_ms;
            scalar_predicate_expression!(predicate, expr, ifnull(expr, 0i64), |v| v
                .unix_timestamp_millis())
        }
    }
}

fn build_condition_filter_expression(
    filter: ConditionFilter,
) -> TrackSearchExpressionBoxed<'static> {
    match filter {
        ConditionFilter::SourceTracked => Box::new(
            view_track_search::media_source_id
                .eq_any(media_tracker_source::table.select(media_tracker_source::source_id)),
        ),
        ConditionFilter::SourceUntracked => Box::new(
            view_track_search::media_source_id
                .ne_all(media_tracker_source::table.select(media_tracker_source::source_id)),
        ),
    }
}

#[expect(clippy::too_many_lines)] // TODO
fn select_track_ids_matching_tag_filter(
    filter: &TagFilter,
) -> (
    track_tag::BoxedQuery<'_, DbBackend, sql_types::BigInt>,
    Option<FilterModifier>,
) {
    let mut select = track_tag::table.select(track_tag::track_id).into_boxed();

    let TagFilter {
        modifier,
        facets,
        label,
        score,
    } = filter;

    // Filter tag facet(s)
    if let Some(filter) = facets {
        match filter {
            FacetsFilter::Prefix(prefix) => {
                let prefix = prefix.as_str();
                if !prefix.is_empty() {
                    let sql_prefix_filter = sql_column_substr_prefix_eq("track_tag.facet", prefix);
                    select = select.filter(sql_prefix_filter);
                }
            }
            FacetsFilter::AnyOf(any_of) => {
                if any_of.is_empty() {
                    select = select.filter(track_tag::facet.is_null());
                } else {
                    // Tags with any of the given facets.
                    let mut count_non_default = 0;
                    let any_of_non_default =
                        any_of.iter().filter(|&key| *key != FacetKey::unfaceted());
                    select =
                        select.filter(track_tag::facet.eq_any(any_of_non_default.map(|key| {
                            count_non_default += 1;
                            key.as_str()
                        })));
                    debug_assert!(count_non_default <= any_of.len());
                    if count_non_default < any_of.len() {
                        // Include unfaceted tags without a facet.
                        select = select.or_filter(track_tag::facet.is_null());
                    } else {
                        // Explicitly exclude unfaceted tags without a facet.
                        // This additional condition is required because the nullable column!
                        select = select.filter(diesel::dsl::not(track_tag::facet.is_null()));
                    }
                }
            }
            FacetsFilter::NoneOf(none_of) => {
                if none_of.is_empty() {
                    select = select.filter(track_tag::facet.is_not_null());
                } else {
                    // Tags with none of the given facets.
                    let mut count_non_default = 0;
                    let none_of_non_default =
                        none_of.iter().filter(|&key| *key != FacetKey::unfaceted());
                    select =
                        select.filter(track_tag::facet.ne_all(none_of_non_default.map(|key| {
                            count_non_default += 1;
                            key.as_str()
                        })));
                    debug_assert!(count_non_default <= none_of.len());
                    if count_non_default < none_of.len() {
                        // Explicitly exclude unfaceted tags without a facet.
                        // This additional condition is required because the nullable column!
                        select = select.filter(track_tag::facet.is_not_null());
                    } else {
                        // Include unfaceted tags without a facet.
                        select = select.or_filter(track_tag::facet.is_null());
                    }
                }
            }
        }
    }

    // Filter tag labels
    if let Some(label) = label
        && let Some((val, cmp, dir)) = decompose_string_predicate(label)
    {
        let string_cmp_op = StringCmpOp::new(cmp, val);
        match string_cmp_op {
            StringCmpOp::Equal(eq) => {
                if dir {
                    select = select.filter(track_tag::label.eq(eq));
                    // Exclude tags without a label.
                    select = select.filter(diesel::dsl::not(track_tag::label.is_null()));
                } else {
                    select = select.filter(track_tag::label.ne(eq));
                    // Include tags without a label.
                    select = select.or_filter(track_tag::label.is_null());
                }
            }
            StringCmpOp::Prefix(prefix) => {
                let sql_prefix_filter = if dir {
                    sql_column_substr_prefix_eq("track_tag.label", &prefix)
                } else {
                    sql_column_substr_prefix_ne("track_tag.label", &prefix)
                };
                select = select.filter(sql_prefix_filter);
            }
            StringCmpOp::Like { escaped } => {
                if dir {
                    select =
                        select.filter(track_tag::label.like(escaped).escape(LIKE_ESCAPE_CHARACTER));
                } else {
                    select = select.filter(
                        track_tag::label
                            .not_like(escaped)
                            .escape(LIKE_ESCAPE_CHARACTER),
                    );
                }
            }
        }
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

fn build_tag_filter_expression(filter: &TagFilter) -> TrackSearchExpressionBoxed<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_tag_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

fn build_cue_label_filter_expression<'a>(
    filter: &StringFilter<'_>,
) -> TrackSearchExpressionBoxed<'a> {
    let (subselect, filter_modifier) = select_track_ids_matching_cue_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

fn select_track_ids_matching_cue_filter<'db>(
    filter: &StringFilter<'_>,
) -> (
    track_cue::BoxedQuery<'db, DbBackend, sql_types::BigInt>,
    Option<FilterModifier>,
) {
    let mut select = track_cue::table.select(track_cue::track_id).into_boxed();

    // Filter cue labels
    if let Some(label) = &filter.value
        && let Some((val, cmp, dir)) = decompose_string_predicate(label)
    {
        let string_cmp_op = StringCmpOp::new(cmp, val);
        select = match string_cmp_op {
            StringCmpOp::Equal(eq) => {
                if dir {
                    select.filter(track_cue::label.eq(eq))
                } else {
                    select.filter(track_cue::label.ne(eq))
                }
            }
            StringCmpOp::Prefix(prefix) => {
                let sql_prefix_filter = if dir {
                    sql_column_substr_prefix_eq("track_cue.label", &prefix)
                } else {
                    sql_column_substr_prefix_ne("track_cue.label", &prefix)
                };
                select.filter(sql_prefix_filter)
            }
            StringCmpOp::Like { escaped } => {
                if dir {
                    select.filter(track_cue::label.like(escaped).escape(LIKE_ESCAPE_CHARACTER))
                } else {
                    select.filter(
                        track_cue::label
                            .not_like(escaped)
                            .escape(LIKE_ESCAPE_CHARACTER),
                    )
                }
            }
        };
    }

    (select, filter.modifier)
}

fn build_any_playlist_uid_filter_expression(
    any_playlist_uid: &[PlaylistUid],
) -> TrackSearchExpressionBoxed<'_> {
    let subselect = select_track_ids_matching_any_playlist_uid_filter(any_playlist_uid);
    Box::new(view_track_search::row_id.eq_any(subselect))
}

fn select_track_ids_matching_any_playlist_uid_filter<'db>(
    any_playlist_uid: impl IntoIterator<Item = &'db PlaylistUid>,
) -> view_track_search::BoxedQuery<'db, DbBackend, sql_types::BigInt> {
    let subselect = playlist::table
        .inner_join(playlist_entry::table)
        .select(playlist_entry::track_id)
        .filter(
            playlist::entity_uid.eq_any(
                any_playlist_uid
                    .into_iter()
                    .map(|entity_uid| encode_entity_uid(entity_uid).to_string()),
            ),
        )
        .filter(playlist_entry::track_id.is_not_null());
    view_track_search::table
        .select(view_track_search::row_id)
        .filter(view_track_search::row_id.nullable().eq_any(subselect))
        .into_boxed()
}

fn select_track_ids_matching_actor_filter(
    filter: &ActorPhraseFilter,
) -> (
    track_actor::BoxedQuery<'_, DbBackend, sql_types::BigInt>,
    Option<FilterModifier>,
) {
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
        select = select.filter(track_actor::scope.eq(encode_search_scope(*scope)));
    }

    // Filter role(s)
    if !roles.is_empty() {
        select = select.filter(
            track_actor::role.eq_any(
                roles
                    .iter()
                    .copied()
                    .map(crate::db::track_actor::encode_role),
            ),
        );
    }

    // Filter kind(s)
    if !kinds.is_empty() {
        select = select.filter(
            track_actor::kind.eq_any(
                kinds
                    .iter()
                    .copied()
                    .map(crate::db::track_actor::encode_kind),
            ),
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
    }

    (select, *modifier)
}

fn build_actor_filter_expression(filter: &ActorPhraseFilter) -> TrackSearchExpressionBoxed<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_actor_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

fn select_track_ids_matching_title_filter(
    filter: &TitlePhraseFilter,
) -> (
    track_title::BoxedQuery<'_, DbBackend, sql_types::BigInt>,
    Option<FilterModifier>,
) {
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
        select = select.filter(track_title::scope.eq(encode_search_scope(*scope)));
    }

    // Filter kind(s)
    if !kinds.is_empty() {
        select = select.filter(
            track_title::kind.eq_any(
                kinds
                    .iter()
                    .copied()
                    .map(crate::db::track_title::encode_kind),
            ),
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
    }

    (select, *modifier)
}

fn build_title_filter_expression(filter: &TitlePhraseFilter) -> TrackSearchExpressionBoxed<'_> {
    let (subselect, filter_modifier) = select_track_ids_matching_title_filter(filter);
    match filter_modifier {
        None => Box::new(view_track_search::row_id.eq_any(subselect)),
        Some(FilterModifier::Complement) => Box::new(view_track_search::row_id.ne_all(subselect)),
    }
}

pub(crate) fn build_filter_expression(filter: &TrackFilter) -> TrackSearchExpressionBoxed<'_> {
    use TrackFilter::*;
    match filter {
        Phrase(filter) => build_phrase_field_filter_expression(filter),
        Numeric(filter) => build_numeric_field_filter_expression(filter),
        DateTime(filter) => build_datetime_field_filter_expression(filter),
        Condition(filter) => build_condition_filter_expression(*filter),
        Tag(filter) => build_tag_filter_expression(filter),
        CueLabel(filter) => build_cue_label_filter_expression(filter),
        AnyTrackUid(any_track_uid) => build_any_track_uid_filter_expression(any_track_uid),
        AnyPlaylistUid(any_playlist_uid) => {
            build_any_playlist_uid_filter_expression(any_playlist_uid)
        }
        ActorPhrase(filter) => build_actor_filter_expression(filter),
        TitlePhrase(filter) => build_title_filter_expression(filter),
        All(filters) => filters
            .iter()
            .fold(dummy_true_expression(), |expr, filter| {
                Box::new(expr.and(build_filter_expression(filter)))
            }),
        Any(filters) => filters
            .iter()
            .fold(dummy_false_expression(), |expr, filter| {
                Box::new(expr.or(build_filter_expression(filter)))
            }),
        Not(filter) => Box::new(diesel::dsl::not(build_filter_expression(filter))),
    }
}

#[cfg(test)]
mod tests;
