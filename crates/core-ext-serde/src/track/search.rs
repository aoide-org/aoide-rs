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

use url::Url;

use crate::{
    _inner::filtering::NumericValue,
    filtering::{ScalarFieldFilter, StringFilter},
    prelude::*,
    sorting::SortDirection,
    tag::search::Filter as TagFilter,
    Pagination,
};

mod _inner {
    pub use crate::_inner::{filtering::*, sorting::*, track::search::*};
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
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

#[cfg(feature = "backend")]
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

#[cfg(feature = "frontend")]
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

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SortOrder(SortField, SortDirection);

#[cfg(feature = "backend")]
impl From<SortOrder> for _inner::SortOrder {
    fn from(from: SortOrder) -> Self {
        let SortOrder(field, direction) = from;
        Self {
            field: field.into(),
            direction: direction.into(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::SortOrder> for SortOrder {
    fn from(from: _inner::SortOrder) -> Self {
        let _inner::SortOrder { field, direction } = from;
        Self(field.into(), direction.into())
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
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

#[cfg(feature = "backend")]
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

#[cfg(feature = "frontend")]
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

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
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

#[cfg(feature = "backend")]
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

#[cfg(feature = "frontend")]
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

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub enum DateTimeField {
    LastPlayedAt,
    ReleasedAt,
    SourceCollectedAt,
    SourceSynchronizedAt,
}

#[cfg(feature = "backend")]
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

#[cfg(feature = "frontend")]
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

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
}

#[cfg(feature = "backend")]
impl From<ConditionFilter> for _inner::ConditionFilter {
    fn from(from: ConditionFilter) -> Self {
        use ConditionFilter::*;
        match from {
            SourceTracked => Self::SourceTracked,
            SourceUntracked => Self::SourceUntracked,
        }
    }
}

#[cfg(feature = "frontend")]
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

#[cfg(feature = "backend")]
impl From<NumericFieldFilter> for _inner::NumericFieldFilter {
    fn from(from: NumericFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::NumericFieldFilter> for NumericFieldFilter {
    fn from(from: _inner::NumericFieldFilter) -> Self {
        let _inner::ScalarFieldFilter { field, predicate } = from;
        Self(field.into(), predicate.into())
    }
}

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

#[cfg(feature = "backend")]
impl From<DateTimeFieldFilter> for _inner::DateTimeFieldFilter {
    fn from(from: DateTimeFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::DateTimeFieldFilter> for DateTimeFieldFilter {
    fn from(from: _inner::DateTimeFieldFilter) -> Self {
        let _inner::ScalarFieldFilter { field, predicate } = from;
        Self(field.into(), predicate.into())
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct PhraseFieldFilter(Vec<StringField>, Vec<String>);

#[cfg(feature = "backend")]
impl From<PhraseFieldFilter> for _inner::PhraseFieldFilter {
    fn from(from: PhraseFieldFilter) -> Self {
        let PhraseFieldFilter(fields, terms) = from;
        Self {
            fields: fields.into_iter().map(Into::into).collect(),
            terms,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::PhraseFieldFilter> for PhraseFieldFilter {
    fn from(from: _inner::PhraseFieldFilter) -> Self {
        let _inner::PhraseFieldFilter { fields, terms } = from;
        Self(fields.into_iter().map(Into::into).collect(), terms)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
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

#[cfg(feature = "backend")]
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

#[cfg(feature = "frontend")]
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

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_url_from_path: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
    // TODO: Replace separate limit/offset properties with flattened
    // pagination after serde issue has been fixed:
    // https://github.com/serde-rs/serde/issues/1183
    //#[serde(flatten)]
    //pub pagination: Pagination,
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<SearchFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

#[cfg(feature = "frontend")]
pub fn client_request_params(
    params: _inner::Params,
    pagination: impl Into<Pagination>,
) -> (QueryParams, SearchParams) {
    let _inner::Params {
        override_root_url,
        filter,
        ordering,
        resolve_url_from_path,
    } = params;
    let Pagination { limit, offset } = pagination.into();
    let query_params = QueryParams {
        limit,
        offset,
        resolve_url_from_path: Some(resolve_url_from_path),
        override_root_url: override_root_url.map(Into::into),
    };
    let search_params = SearchParams {
        filter: filter.map(Into::into),
        ordering: ordering.into_iter().map(Into::into).collect(),
    };
    (query_params, search_params)
}