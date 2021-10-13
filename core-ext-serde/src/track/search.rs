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

use aoide_core_serde::{entity::EntityUid, util::clock::DateTime};

use crate::{
    _inner::filtering::NumericValue,
    filtering::{ScalarFieldFilter, StringFilter},
    prelude::*,
    sorting::SortDirection,
    tag::search::Filter as TagFilter,
};

mod _inner {
    pub use crate::_inner::{filtering::*, sorting::*, track::search::*};
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    AlbumArtist,
    AlbumTitle,
    AudioBitrateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioLoudnessLufs,
    AudioSampleRateHz,
    CreatedAt,
    DiscNumber,
    DiscTotal,
    LastPlayedAt,
    MusicTempoBpm,
    MusicKeyCode,
    ReleasedAtDate,
    ReleasedBy,
    SourceCollectedAt,
    SourceSynchronizedAt,
    SourceType,
    SourcePath,
    TrackArtist,
    TrackNumber,
    TrackTitle,
    TrackTotal,
    TimesPlayed,
    UpdatedAt,
}

impl From<SortField> for _inner::SortField {
    fn from(from: SortField) -> Self {
        use SortField::*;
        match from {
            AlbumArtist => Self::AlbumArtist,
            AlbumTitle => Self::AlbumTitle,
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            CreatedAt => Self::CreatedAt,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            LastPlayedAt => Self::LastPlayedAt,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
            ReleasedAtDate => Self::ReleasedAtDate,
            ReleasedBy => Self::ReleasedBy,
            SourceCollectedAt => Self::SourceCollectedAt,
            SourceSynchronizedAt => Self::SourceSynchronizedAt,
            SourcePath => Self::SourcePath,
            SourceType => Self::SourceType,
            TimesPlayed => Self::TimesPlayed,
            TrackArtist => Self::TrackArtist,
            TrackNumber => Self::TrackNumber,
            TrackTitle => Self::TrackTitle,
            TrackTotal => Self::TrackTotal,
            UpdatedAt => Self::UpdatedAt,
        }
    }
}

impl From<_inner::SortField> for SortField {
    fn from(from: _inner::SortField) -> Self {
        use _inner::SortField::*;
        match from {
            AlbumArtist => Self::AlbumArtist,
            AlbumTitle => Self::AlbumTitle,
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            CreatedAt => Self::CreatedAt,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            LastPlayedAt => Self::LastPlayedAt,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
            ReleasedAtDate => Self::ReleasedAtDate,
            ReleasedBy => Self::ReleasedBy,
            SourceCollectedAt => Self::SourceCollectedAt,
            SourceSynchronizedAt => Self::SourceSynchronizedAt,
            SourcePath => Self::SourcePath,
            SourceType => Self::SourceType,
            TimesPlayed => Self::TimesPlayed,
            TrackArtist => Self::TrackArtist,
            TrackNumber => Self::TrackNumber,
            TrackTitle => Self::TrackTitle,
            TrackTotal => Self::TrackTotal,
            UpdatedAt => Self::UpdatedAt,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct SortOrder(SortField, SortDirection);

impl From<SortOrder> for _inner::SortOrder {
    fn from(from: SortOrder) -> Self {
        let SortOrder(field, direction) = from;
        Self {
            field: field.into(),
            direction: direction.into(),
        }
    }
}

impl From<_inner::SortOrder> for SortOrder {
    fn from(from: _inner::SortOrder) -> Self {
        let _inner::SortOrder { field, direction } = from;
        Self(field.into(), direction.into())
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringField {
    AlbumArtist,
    AlbumTitle,
    SourceType,
    SourcePath,
    TrackArtist,
    TrackComposer,
    TrackTitle,
    ReleasedBy,
}

impl From<StringField> for _inner::StringField {
    fn from(from: StringField) -> Self {
        use StringField::*;
        match from {
            AlbumArtist => Self::AlbumArtist,
            AlbumTitle => Self::AlbumTitle,
            SourceType => Self::SourceType,
            SourcePath => Self::SourcePath,
            TrackArtist => Self::TrackArtist,
            TrackComposer => Self::TrackComposer,
            TrackTitle => Self::TrackTitle,
            ReleasedBy => Self::ReleasedBy,
        }
    }
}

impl From<_inner::StringField> for StringField {
    fn from(from: _inner::StringField) -> Self {
        use _inner::StringField::*;
        match from {
            AlbumArtist => Self::AlbumArtist,
            AlbumTitle => Self::AlbumTitle,
            SourceType => Self::SourceType,
            SourcePath => Self::SourcePath,
            TrackArtist => Self::TrackArtist,
            TrackComposer => Self::TrackComposer,
            TrackTitle => Self::TrackTitle,
            ReleasedBy => Self::ReleasedBy,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumericField {
    AdvisoryRating,
    AudioBitrateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioSampleRateHz,
    AudioLoudnessLufs,
    DiscNumber,
    DiscTotal,
    ReleasedAtDate,
    MusicTempoBpm,
    MusicKeyCode,
    TimesPlayed,
    TrackNumber,
    TrackTotal,
}

impl From<NumericField> for _inner::NumericField {
    fn from(from: NumericField) -> Self {
        use NumericField::*;
        match from {
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            AdvisoryRating => Self::AdvisoryRating,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
            ReleasedAtDate => Self::ReleasedAtDate,
            TrackNumber => Self::TrackNumber,
            TrackTotal => Self::TrackTotal,
            TimesPlayed => Self::TimesPlayed,
        }
    }
}

impl From<_inner::NumericField> for NumericField {
    fn from(from: _inner::NumericField) -> Self {
        use _inner::NumericField::*;
        match from {
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            AdvisoryRating => Self::AdvisoryRating,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
            ReleasedAtDate => Self::ReleasedAtDate,
            TimesPlayed => Self::TimesPlayed,
            TrackNumber => Self::TrackNumber,
            TrackTotal => Self::TrackTotal,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DateTimeField {
    LastPlayedAt,
    ReleasedAt,
    SourceCollectedAt,
    SourceSynchronizedAt,
}

impl From<DateTimeField> for _inner::DateTimeField {
    fn from(from: DateTimeField) -> Self {
        use DateTimeField::*;
        match from {
            LastPlayedAt => Self::LastPlayedAt,
            ReleasedAt => Self::ReleasedAt,
            SourceCollectedAt => Self::SourceCollectedAt,
            SourceSynchronizedAt => Self::SourceSynchronizedAt,
        }
    }
}

impl From<_inner::DateTimeField> for DateTimeField {
    fn from(from: _inner::DateTimeField) -> Self {
        use _inner::DateTimeField::*;
        match from {
            LastPlayedAt => Self::LastPlayedAt,
            ReleasedAt => Self::ReleasedAt,
            SourceCollectedAt => Self::SourceCollectedAt,
            SourceSynchronizedAt => Self::SourceSynchronizedAt,
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
}

impl From<ConditionFilter> for _inner::ConditionFilter {
    fn from(from: ConditionFilter) -> Self {
        use ConditionFilter::*;
        match from {
            SourceTracked => Self::SourceTracked,
            SourceUntracked => Self::SourceUntracked,
        }
    }
}

impl From<_inner::ConditionFilter> for ConditionFilter {
    fn from(from: _inner::ConditionFilter) -> Self {
        use _inner::ConditionFilter::*;
        match from {
            SourceTracked => Self::SourceTracked,
            SourceUntracked => Self::SourceUntracked,
        }
    }
}

pub type NumericFieldFilter = ScalarFieldFilter<NumericField, NumericValue>;

impl From<NumericFieldFilter> for _inner::NumericFieldFilter {
    fn from(from: NumericFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

impl From<_inner::NumericFieldFilter> for NumericFieldFilter {
    fn from(from: _inner::NumericFieldFilter) -> Self {
        let _inner::ScalarFieldFilter { field, predicate } = from;
        Self(field.into(), predicate.into())
    }
}

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

impl From<DateTimeFieldFilter> for _inner::DateTimeFieldFilter {
    fn from(from: DateTimeFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

impl From<_inner::DateTimeFieldFilter> for DateTimeFieldFilter {
    fn from(from: _inner::DateTimeFieldFilter) -> Self {
        let _inner::ScalarFieldFilter { field, predicate } = from;
        Self(field.into(), predicate.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhraseFieldFilter(Vec<StringField>, Vec<String>);

impl From<PhraseFieldFilter> for _inner::PhraseFieldFilter {
    fn from(from: PhraseFieldFilter) -> Self {
        let PhraseFieldFilter(fields, terms) = from;
        Self {
            fields: fields.into_iter().map(Into::into).collect(),
            terms,
        }
    }
}

impl From<_inner::PhraseFieldFilter> for PhraseFieldFilter {
    fn from(from: _inner::PhraseFieldFilter) -> Self {
        let _inner::PhraseFieldFilter { fields, terms } = from;
        Self(fields.into_iter().map(Into::into).collect(), terms)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchFilter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Condition(ConditionFilter),
    Tag(TagFilter),
    CueLabel(StringFilter),
    PlaylistUid(EntityUid),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

impl From<SearchFilter> for _inner::SearchFilter {
    fn from(from: SearchFilter) -> Self {
        use SearchFilter::*;
        match from {
            Phrase(from) => Self::Phrase(from.into()),
            Numeric(from) => Self::Numeric(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
            Condition(from) => Self::Condition(from.into()),
            Tag(from) => Self::Tag(from.into()),
            CueLabel(from) => Self::CueLabel(from.into()),
            PlaylistUid(from) => Self::PlaylistUid(from.into()),
            All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

impl From<_inner::SearchFilter> for SearchFilter {
    fn from(from: _inner::SearchFilter) -> Self {
        use _inner::SearchFilter::*;
        match from {
            Phrase(from) => Self::Phrase(from.into()),
            Numeric(from) => Self::Numeric(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
            Condition(from) => Self::Condition(from.into()),
            Tag(from) => Self::Tag(from.into()),
            CueLabel(from) => Self::CueLabel(from.into()),
            PlaylistUid(from) => Self::PlaylistUid(from.into()),
            All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<SearchFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

impl From<SearchParams> for _inner::SearchParams {
    fn from(from: SearchParams) -> Self {
        Self {
            filter: from.filter.map(Into::into),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<_inner::SearchParams> for SearchParams {
    fn from(from: _inner::SearchParams) -> Self {
        Self {
            filter: from.filter.map(Into::into),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}
