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

use super::*;

mod uc {
    pub use crate::usecases::tracks::search::*;
}

mod _repo {
    pub use aoide_repo::{
        prelude::*,
        tag::Filter as TagFilter,
        track::{
            DateTimeField, DateTimeFieldFilter, NumericField, NumericFieldFilter,
            PhraseFieldFilter, SearchFilter, SearchParams, SortField, SortOrder, StringField,
        },
    };
}

use aoide_repo::prelude::NumericValue;

use aoide_core_serde::{track::Entity, util::clock::DateTime};

///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    AlbumArtist,
    AlbumTitle,
    AudioBitRate,
    AudioChannelCount,
    AudioDuration,
    AudioLoudness,
    AudioSampleRate,
    CreatedAt,
    DiscNumber,
    DiscTotal,
    LastPlayedAt,
    MusicTempo,
    MusicKey,
    ReleaseDate,
    SourceCollectedAt,
    SourceSynchronizedAt,
    SourceUri,
    SourceUriDecoded,
    TrackArtist,
    TrackNumber,
    TrackTitle,
    TrackTotal,
    TimesPlayed,
    UpdatedAt,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "desc")]
    Descending,
}

impl From<SortDirection> for _repo::SortDirection {
    fn from(from: SortDirection) -> Self {
        use _repo::SortDirection::*;
        match from {
            SortDirection::Ascending => Ascending,
            SortDirection::Descending => Descending,
        }
    }
}

impl From<SortField> for _repo::SortField {
    fn from(from: SortField) -> Self {
        use SortField::*;
        match from {
            AlbumArtist => Self::AlbumArtist,
            AlbumTitle => Self::AlbumTitle,
            AudioBitRate => Self::AudioBitRate,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDuration => Self::AudioDuration,
            AudioLoudness => Self::AudioLoudness,
            AudioSampleRate => Self::AudioSampleRate,
            CreatedAt => Self::CreatedAt,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            LastPlayedAt => Self::LastPlayedAt,
            MusicTempo => Self::MusicTempo,
            MusicKey => Self::MusicKey,
            ReleaseDate => Self::ReleaseDate,
            SourceCollectedAt => Self::SourceCollectedAt,
            SourceSynchronizedAt => Self::SourceSynchronizedAt,
            SourceUri => Self::SourceUri,
            SourceUriDecoded => Self::SourceUriDecoded,
            TimesPlayed => Self::TimesPlayed,
            TrackArtist => Self::TrackArtist,
            TrackNumber => Self::TrackNumber,
            TrackTitle => Self::TrackTitle,
            TrackTotal => Self::TrackTotal,
            UpdatedAt => Self::UpdatedAt,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct SortOrder(SortField, SortDirection);

impl From<SortOrder> for _repo::SortOrder {
    fn from(from: SortOrder) -> Self {
        Self {
            field: from.0.into(),
            direction: from.1.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FilterModifier {
    Complement,
}

impl From<FilterModifier> for _repo::FilterModifier {
    fn from(from: FilterModifier) -> Self {
        use _repo::FilterModifier::*;
        match from {
            FilterModifier::Complement => Complement,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFilter {
    #[serde(skip_serializing_if = "Option::None")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::None")]
    pub value: Option<StringPredicate>,
}

impl From<StringFilter> for _repo::StringFilter {
    fn from(from: StringFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            value: from.value.map(Into::into),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StringField {
    AlbumArtist,
    AlbumTitle,
    MediaType,
    SourceUri, // percent-encoded URI
    SourceUriDecoded,
    TrackArtist,
    TrackComposer,
    TrackTitle,
}

impl From<StringField> for _repo::StringField {
    fn from(from: StringField) -> Self {
        use StringField::*;
        match from {
            AlbumArtist => Self::AlbumArtist,
            AlbumTitle => Self::AlbumTitle,
            MediaType => Self::MediaType,
            SourceUri => Self::SourceUri,
            SourceUriDecoded => Self::SourceUriDecoded,
            TrackArtist => Self::TrackArtist,
            TrackComposer => Self::TrackComposer,
            TrackTitle => Self::TrackTitle,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumericField {
    AudioBitRateBps,
    AudioChannelCount,
    AudioDurationMs,
    AudioSampleRateHz,
    AudioLoudnessLufs,
    DiscNumber,
    DiscTotal,
    ReleaseDate,
    MusicTempoBpm,
    MusicKeyCode,
    TrackNumber,
    TrackTotal,
}

impl From<NumericField> for _repo::NumericField {
    fn from(from: NumericField) -> Self {
        use NumericField::*;
        match from {
            AudioBitRateBps => Self::AudioBitRateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            TrackNumber => Self::TrackNumber,
            TrackTotal => Self::TrackTotal,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            ReleaseDate => Self::ReleaseDate,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DateTimeField {
    LastPlayedAt,
    ReleasedAt,
    SourceCollectedAt,
    SourceSynchronizedAt,
}

impl From<DateTimeField> for _repo::DateTimeField {
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

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum ScalarPredicate<V> {
    #[serde(rename = "lt")]
    LessThan(V),

    #[serde(rename = "le")]
    LessOrEqual(V),

    #[serde(rename = "gt")]
    GreaterThan(V),

    #[serde(rename = "ge")]
    GreaterOrEqual(V),

    #[serde(rename = "eq")]
    Equal(Option<V>),

    #[serde(rename = "ne")]
    NotEqual(Option<V>),
}

pub type NumericPredicate = ScalarPredicate<_repo::NumericValue>;

impl From<NumericPredicate> for _repo::NumericPredicate {
    fn from(from: NumericPredicate) -> Self {
        match from {
            ScalarPredicate::LessThan(val) => Self::LessThan(val),
            ScalarPredicate::LessOrEqual(val) => Self::LessOrEqual(val),
            ScalarPredicate::GreaterThan(val) => Self::GreaterThan(val),
            ScalarPredicate::GreaterOrEqual(val) => Self::GreaterOrEqual(val),
            ScalarPredicate::Equal(val) => Self::Equal(val),
            ScalarPredicate::NotEqual(val) => Self::NotEqual(val),
        }
    }
}

pub type DateTimePredicate = ScalarPredicate<DateTime>;

impl From<DateTimePredicate> for _repo::DateTimePredicate {
    fn from(from: DateTimePredicate) -> Self {
        match from {
            ScalarPredicate::LessThan(val) => Self::LessThan(val.into()),
            ScalarPredicate::LessOrEqual(val) => Self::LessOrEqual(val.into()),
            ScalarPredicate::GreaterThan(val) => Self::GreaterThan(val.into()),
            ScalarPredicate::GreaterOrEqual(val) => Self::GreaterOrEqual(val.into()),
            ScalarPredicate::Equal(val) => Self::Equal(val.map(Into::into)),
            ScalarPredicate::NotEqual(val) => Self::NotEqual(val.map(Into::into)),
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct ScalarFieldFilter<F, V>(F, ScalarPredicate<V>);

pub type NumericFieldFilter = ScalarFieldFilter<NumericField, NumericValue>;

impl From<NumericFieldFilter> for _repo::NumericFieldFilter {
    fn from(from: NumericFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

pub type DateTimeFieldFilter = ScalarFieldFilter<DateTimeField, DateTime>;

impl From<DateTimeFieldFilter> for _repo::DateTimeFieldFilter {
    fn from(from: DateTimeFieldFilter) -> Self {
        let ScalarFieldFilter(field, predicate) = from;
        Self {
            field: field.into(),
            predicate: predicate.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PhraseFieldFilter(Vec<StringField>, Vec<String>);

impl From<PhraseFieldFilter> for _repo::PhraseFieldFilter {
    fn from(from: PhraseFieldFilter) -> Self {
        let PhraseFieldFilter(fields, terms) = from;
        Self {
            fields: fields.into_iter().map(Into::into).collect(),
            terms,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFilter {
    #[serde(skip_serializing_if = "Option::None")]
    pub modifier: Option<FilterModifier>,

    // Facets are always matched with equals. Use an empty vector
    // for matching only tags without a facet.
    #[serde(skip_serializing_if = "Option::None")]
    pub facets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::None")]
    pub label: Option<StringPredicate>,

    #[serde(skip_serializing_if = "Option::None")]
    pub score: Option<NumericPredicate>,
}

impl From<TagFilter> for _repo::TagFilter {
    fn from(from: TagFilter) -> Self {
        Self {
            modifier: from.modifier.map(Into::into),
            facets: from.facets,
            label: from.label.map(Into::into),
            score: from.score.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SearchFilter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Tag(TagFilter),
    CueLabel(StringFilter),
    All(Vec<SearchFilter>),
    Any(Vec<SearchFilter>),
    Not(Box<SearchFilter>),
}

impl From<SearchFilter> for _repo::SearchFilter {
    fn from(from: SearchFilter) -> Self {
        use SearchFilter::*;
        match from {
            Phrase(from) => Self::Phrase(from.into()),
            Numeric(from) => Self::Numeric(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
            Tag(from) => Self::Tag(from.into()),
            CueLabel(from) => Self::CueLabel(from.into()),
            All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<SearchFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

impl From<RequestBody> for _repo::SearchParams {
    fn from(from: RequestBody) -> Self {
        Self {
            filter: from.filter.map(Into::into),
            ordering: from.ordering.into_iter().map(Into::into).collect(),
        }
    }
}

pub type ResponseBody = Vec<Entity>;

const DEFAULT_PAGINATION: Pagination = Pagination {
    limit: 100,
    offset: None,
};

pub fn handle_request(
    pooled_connection: &SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    query_params: PaginationQueryParams,
    request_body: RequestBody,
) -> RepoResult<ResponseBody> {
    let RequestBody { filter, ordering } = request_body;
    let mut collector = EntityCollector::default();
    uc::search(
        pooled_connection,
        collection_uid,
        &Option::from(query_params).unwrap_or(DEFAULT_PAGINATION),
        filter.map(Into::into),
        ordering.into_iter().map(Into::into).collect(),
        &mut collector,
    )?;
    Ok(collector.into())
}
