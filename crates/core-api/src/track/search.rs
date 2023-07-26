// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    audio::DurationMs,
    track::{
        actor::{Kind as ActorKind, Role as ActorRole},
        title::Kind as TitleKind,
    },
    util::clock::{DateOrDateTime, OffsetDateTimeMs},
    PlaylistUid, TrackUid,
};
use strum::FromRepr;

use crate::{filtering::*, media::source::ResolveUrlFromContentPath, sorting::*, tag};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StringField {
    ContentPath,
    ContentType, // RFC 6838 media type
    Copyright,
    Publisher,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NumericField {
    AudioBitrateBps,
    AudioChannelCount,
    AudioChannelMask,
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

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, OffsetDateTimeMs>;

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
    pub path: StringPredicate<'s>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum Scope {
    Track = 0,
    Album = 1,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActorPhraseFilter {
    pub modifier: Option<FilterModifier>,

    /// The given scope or any scope if `None`.
    pub scope: Option<Scope>,

    /// Any of the given roles or any role if empty.
    pub roles: Vec<ActorRole>,

    /// Any of the given kinds or any kind if empty.
    pub kinds: Vec<ActorKind>,

    /// Name that matches all of the given terms in order or any name if empty.
    pub name_terms: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TitlePhraseFilter {
    pub modifier: Option<FilterModifier>,

    /// The given scope or any scope if `None`.
    pub scope: Option<Scope>,

    /// Any of the given kinds or any kind if empty.
    pub kinds: Vec<TitleKind>,

    /// Name that matches all of the given terms in order or any name if empty.
    pub name_terms: Vec<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SortField {
    AudioBitrateBps,
    AudioChannelCount,
    AudioChannelMask,
    AudioDurationMs,
    AudioLoudnessLufs,
    AudioSampleRateHz,
    CollectedAt,
    ContentPath,
    ContentType,
    Copyright,
    CreatedAt,
    DiscNumber,
    DiscTotal,
    MusicTempoBpm,
    MusicKeyCode,
    Publisher,
    RecordedAtDate,
    ReleasedAtDate,
    ReleasedOrigAtDate,
    TrackNumber,
    TrackTotal,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SortOrder {
    pub field: SortField,
    pub direction: SortDirection,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Filter {
    Phrase(PhraseFieldFilter),
    ActorPhrase(ActorPhraseFilter),
    TitlePhrase(TitlePhraseFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Condition(ConditionFilter),
    Tag(tag::search::Filter),
    CueLabel(StringFilter<'static>),
    AnyTrackUid(Vec<TrackUid>),
    AnyPlaylistUid(Vec<PlaylistUid>),
    All(Vec<Filter>),
    Any(Vec<Filter>),
    Not(Box<Filter>),
}

impl Filter {
    #[must_use]
    pub fn recorded_at_equals(recorded_at: DateOrDateTime) -> Self {
        match recorded_at {
            DateOrDateTime::DateTime(dt) => Self::DateTime(DateTimeFieldFilter {
                field: DateTimeField::RecordedAt,
                predicate: DateTimePredicate::Equal(Some(dt)),
            }),
            DateOrDateTime::Date(date) => Self::Numeric(NumericFieldFilter {
                field: NumericField::RecordedAtDate,
                predicate: NumericPredicate::Equal(Some(date.value().into())),
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
                predicate: NumericPredicate::Equal(Some(date.value().into())),
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
                predicate: NumericPredicate::Equal(Some(date.value().into())),
            }),
        }
    }

    #[must_use]
    pub fn audio_duration_around(duration: DurationMs, epsilon: DurationMs) -> Self {
        debug_assert!(duration.is_valid());
        debug_assert!(epsilon.is_valid());
        let duration_value = duration.value();
        let epsilon_value = epsilon.value();
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
    pub filter: Option<Filter>,
    pub ordering: Vec<SortOrder>,
}
