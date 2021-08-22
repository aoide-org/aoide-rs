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

use aoide_core::usecases::filtering::NumericValue;

use crate::{
    entity::EntityUid,
    prelude::*,
    usecases::{filtering::*, sorting::*, tags::search::Filter as TagFilter},
};

mod _core {
    pub use aoide_core::usecases::{filtering::*, sorting::*, tracks::search::*};
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

impl From<SortField> for _core::SortField {
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

impl From<_core::SortField> for SortField {
    fn from(from: _core::SortField) -> Self {
        use _core::SortField::*;
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

impl From<SortOrder> for _core::SortOrder {
    fn from(from: SortOrder) -> Self {
        let SortOrder(field, direction) = from;
        Self {
            field: field.into(),
            direction: direction.into(),
        }
    }
}

impl From<_core::SortOrder> for SortOrder {
    fn from(from: _core::SortOrder) -> Self {
        let _core::SortOrder { field, direction } = from;
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

impl From<StringField> for _core::StringField {
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

impl From<_core::StringField> for StringField {
    fn from(from: _core::StringField) -> Self {
        use _core::StringField::*;
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

impl From<NumericField> for _core::NumericField {
    fn from(from: NumericField) -> Self {
        use NumericField::*;
        match from {
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
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

impl From<_core::NumericField> for NumericField {
    fn from(from: _core::NumericField) -> Self {
        use _core::NumericField::*;
        match from {
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
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

impl From<DateTimeField> for _core::DateTimeField {
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

impl From<_core::DateTimeField> for DateTimeField {
    fn from(from: _core::DateTimeField) -> Self {
        use _core::DateTimeField::*;
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

impl From<ConditionFilter> for _core::ConditionFilter {
    fn from(from: ConditionFilter) -> Self {
        use ConditionFilter::*;
        match from {
            SourceTracked => Self::SourceTracked,
            SourceUntracked => Self::SourceUntracked,
        }
    }
}

impl From<_core::ConditionFilter> for ConditionFilter {
    fn from(from: _core::ConditionFilter) -> Self {
        use _core::ConditionFilter::*;
        match from {
            SourceTracked => Self::SourceTracked,
            SourceUntracked => Self::SourceUntracked,
        }
    }
}

pub type NumericFieldFilter = ScalarFieldFilter<NumericField, NumericValue>;

impl From<NumericFieldFilter> for _core::NumericFieldFilter {
    fn from(from: NumericFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

impl From<_core::NumericFieldFilter> for NumericFieldFilter {
    fn from(from: _core::NumericFieldFilter) -> Self {
        let _core::ScalarFieldFilter { field, predicate } = from;
        Self(field.into(), predicate.into())
    }
}

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

impl From<DateTimeFieldFilter> for _core::DateTimeFieldFilter {
    fn from(from: DateTimeFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

impl From<_core::DateTimeFieldFilter> for DateTimeFieldFilter {
    fn from(from: _core::DateTimeFieldFilter) -> Self {
        let _core::ScalarFieldFilter { field, predicate } = from;
        Self(field.into(), predicate.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PhraseFieldFilter(Vec<StringField>, Vec<String>);

impl From<PhraseFieldFilter> for _core::PhraseFieldFilter {
    fn from(from: PhraseFieldFilter) -> Self {
        let PhraseFieldFilter(fields, terms) = from;
        Self {
            fields: fields.into_iter().map(Into::into).collect(),
            terms,
        }
    }
}

impl From<_core::PhraseFieldFilter> for PhraseFieldFilter {
    fn from(from: _core::PhraseFieldFilter) -> Self {
        let _core::PhraseFieldFilter { fields, terms } = from;
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

impl From<SearchFilter> for _core::SearchFilter {
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

impl From<_core::SearchFilter> for SearchFilter {
    fn from(from: _core::SearchFilter) -> Self {
        use _core::SearchFilter::*;
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

impl From<SearchParams> for _core::SearchParams {
    fn from(from: SearchParams) -> Self {
        Self {
            filter: from.filter.map(Into::into),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<_core::SearchParams> for SearchParams {
    fn from(from: _core::SearchParams) -> Self {
        Self {
            filter: from.filter.map(Into::into),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}
