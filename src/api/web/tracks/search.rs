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
    pub use crate::usecases::tracks::search::search;
    pub use aoide_usecases::tracks::search::Params;
}

mod _repo {
    pub use aoide_repo::{
        prelude::*,
        tag::Filter as TagFilter,
        track::{
            ConditionFilter, DateTimeField, DateTimeFieldFilter, NumericField, NumericFieldFilter,
            PhraseFieldFilter, SearchFilter, SearchParams, SortField, SortOrder, StringField,
        },
    };
}

use aoide_repo::prelude::NumericValue;

use aoide_core_serde::{entity::EntityUid, track::Entity, util::clock::DateTime};

use url::Url;

#[derive(Copy, Clone, Debug, Deserialize)]
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
    ReleaseDate,
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
            ReleaseDate => Self::ReleaseDate,
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
    SourceType,
    SourcePath,
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
            SourceType => Self::SourceType,
            SourcePath => Self::SourcePath,
            TrackArtist => Self::TrackArtist,
            TrackComposer => Self::TrackComposer,
            TrackTitle => Self::TrackTitle,
        }
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NumericField {
    AudioBitrateBps,
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
            AudioBitrateBps => Self::AudioBitrateBps,
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
#[serde(rename_all = "camelCase")]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
}

impl From<ConditionFilter> for _repo::ConditionFilter {
    fn from(from: ConditionFilter) -> Self {
        use ConditionFilter::*;
        match from {
            SourceTracked => Self::SourceTracked,
            SourceUntracked => Self::SourceUntracked,
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
    Condition(ConditionFilter),
    Tag(TagFilter),
    CueLabel(StringFilter),
    PlaylistUid(EntityUid),
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

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_url_from_path: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_base_url: Option<Url>,

    pub limit: Option<PaginationLimit>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,
    // TODO: Replace limit/offset with pagination after serde issue
    // has been fixed: https://github.com/serde-rs/serde/issues/1183
    //#[serde(flatten)]
    //pub pagination: PaginationQueryParams,
}

const DEFAULT_PAGINATION: Pagination = Pagination {
    limit: 100,
    offset: None,
};

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let QueryParams {
        resolve_url_from_path,
        override_base_url,
        limit,
        offset,
    } = query_params;
    let pagination = PaginationQueryParams { limit, offset };
    let pagination = Option::from(pagination).unwrap_or(DEFAULT_PAGINATION);
    // Passing a base URL override implies resolving paths
    let resolve_url_from_path = override_base_url.is_some()
        || resolve_url_from_path.unwrap_or(uc::Params::default().resolve_url_from_path);
    let params = uc::Params {
        override_base_url,
        resolve_url_from_path,
    };
    let RequestBody { filter, ordering } = request_body;
    let mut collector = EntityCollector::default();
    uc::search(
        pooled_connection,
        collection_uid,
        &pagination,
        filter.map(Into::into),
        ordering.into_iter().map(Into::into).collect(),
        params,
        &mut collector,
    )?;
    Ok(collector.into())
}
