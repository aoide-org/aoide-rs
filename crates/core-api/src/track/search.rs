// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use semval::prelude::IsValid as _;

use aoide_core::{
    audio::DurationMs,
    entity::EntityUid,
    util::clock::{DateOrDateTime, DateTime},
};

use crate::{filtering::*, media::source::ResolveUrlFromContentPath, sorting::*, tag};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateTimeField {
    CollectedAt,
    RecordedAt,
    ReleasedAt,
    ReleasedOrigAt,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
}

pub type NumericFieldFilter = ScalarFieldFilter<NumericField, NumericValue>;

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFilterBorrowed<'s> {
    pub path: StringPredicateBorrowed<'s>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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

#[derive(Debug, Clone, Default)]
pub struct Params {
    pub resolve_url_from_content_path: Option<ResolveUrlFromContentPath>,
    pub filter: Option<SearchFilter>,
    pub ordering: Vec<SortOrder>,
}
