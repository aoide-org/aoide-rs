// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use semval::prelude::IsValid as _;

use aoide_core::{
    audio::DurationMs,
    entity::EntityUid,
    util::clock::{DateOrDateTime, DateTime},
};

use crate::{filtering::*, media::source::ResolveUrlFromContentPath, sorting::*, tag};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StringField {
    AlbumArtist,
    AlbumTitle,
    ContentPath,
    ContentType, // RFC 6838 media type
    Publisher,
    TrackArtist,
    TrackComposer,
    TrackTitle,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NumericField {
    AudioBitrateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioLoudnessLufs,
    AudioSampleRateHz,
    AdvisoryRating,
    DiscNumber,
    DiscTotal,
    MusicTempoBpm,
    MusicKeyCode,
    RecordedAtDate,
    ReleasedAtDate,
    ReleasedOrigAtDate,
    TrackNumber,
    TrackTotal,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateTimeField {
    CollectedAt,
    RecordedAt,
    ReleasedAt,
    ReleasedOrigAt,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
}

pub type NumericFieldFilter = ScalarFieldFilter<NumericField, NumericValue>;

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhraseFieldFilter {
    // Empty == All available string fields are considered
    // Disjunction, i.e. a match in one of the fields is sufficient
    pub fields: Vec<StringField>,

    // Concatenated with wildcards and filtered using
    // case-insensitive "contains" semantics against each
    // of the selected fields, e.g. ["pa", "la", "bell"]
    // ["tt, ll"] will both match "Patti LaBelle". An empty
    // argument matches empty as well as missing/null fields.
    pub terms: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceFilterBorrowed<'s> {
    pub path: StringPredicateBorrowed<'s>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SortField {
    AlbumArtist,
    AlbumTitle,
    AudioBitrateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioLoudnessLufs,
    AudioSampleRateHz,
    CollectedAt,
    ContentPath,
    ContentType,
    CreatedAt,
    DiscNumber,
    DiscTotal,
    MusicTempoBpm,
    MusicKeyCode,
    Publisher,
    RecordedAtDate,
    ReleasedAtDate,
    ReleasedOrigAtDate,
    TrackArtist,
    TrackNumber,
    TrackTitle,
    TrackTotal,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SearchFilter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Condition(ConditionFilter),
    Tag(tag::search::Filter),
    CueLabel(StringFilter),
    PlaylistUid(EntityUid),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

impl SearchFilter {
    #[must_use]
    pub fn recorded_at_equals(recorded_at: DateOrDateTime) -> Self {
        match recorded_at {
            DateOrDateTime::DateTime(dt) => Self::DateTime(DateTimeFieldFilter {
                field: DateTimeField::RecordedAt,
                predicate: DateTimePredicate::Equal(Some(dt)),
            }),
            DateOrDateTime::Date(date) => Self::Numeric(NumericFieldFilter {
                field: NumericField::RecordedAtDate,
                predicate: NumericPredicate::Equal(Some(date.to_inner().into())),
            }),
        }
    }

    #[must_use]
    pub fn released_at_equals(released_at: DateOrDateTime) -> Self {
        match released_at {
            DateOrDateTime::DateTime(dt) => Self::DateTime(DateTimeFieldFilter {
                field: DateTimeField::ReleasedAt,
                predicate: DateTimePredicate::Equal(Some(dt)),
            }),
            DateOrDateTime::Date(date) => Self::Numeric(NumericFieldFilter {
                field: NumericField::ReleasedAtDate,
                predicate: NumericPredicate::Equal(Some(date.to_inner().into())),
            }),
        }
    }

    #[must_use]
    pub fn released_orig_at_equals(released_orig_at: DateOrDateTime) -> Self {
        match released_orig_at {
            DateOrDateTime::DateTime(dt) => Self::DateTime(DateTimeFieldFilter {
                field: DateTimeField::ReleasedOrigAt,
                predicate: DateTimePredicate::Equal(Some(dt)),
            }),
            DateOrDateTime::Date(date) => Self::Numeric(NumericFieldFilter {
                field: NumericField::ReleasedOrigAtDate,
                predicate: NumericPredicate::Equal(Some(date.to_inner().into())),
            }),
        }
    }

    #[must_use]
    pub fn audio_duration_around(duration: DurationMs, epsilon: DurationMs) -> Self {
        debug_assert!(duration.is_valid());
        debug_assert!(epsilon.is_valid());
        let duration_value = duration.to_inner();
        let epsilon_value = epsilon.to_inner();
        if epsilon_value > 0.0 {
            Self::All(vec![
                Self::Numeric(NumericFieldFilter {
                    field: NumericField::AudioDurationMs,
                    predicate: NumericPredicate::GreaterOrEqual(
                        (duration_value - epsilon_value).max(0.0),
                    ),
                }),
                Self::Numeric(NumericFieldFilter {
                    field: NumericField::AudioDurationMs,
                    predicate: NumericPredicate::LessOrEqual(duration_value + epsilon_value),
                }),
            ])
        } else {
            Self::Numeric(NumericFieldFilter {
                field: NumericField::AudioDurationMs,
                predicate: NumericPredicate::Equal(Some(duration_value)),
            })
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Params {
    pub resolve_url_from_content_path: Option<ResolveUrlFromContentPath>,
    pub filter: Option<SearchFilter>,
    pub ordering: Vec<SortOrder>,
}
