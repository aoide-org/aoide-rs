// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::entity::EntityUidTyped;
use aoide_core_json::{
    entity::EntityUid,
    track::{
        actor::{Kind as ActorKind, Role as ActorRole},
        title::Kind as TitleKind,
    },
    util::clock::DateTime,
};

use url::Url;

use crate::{
    _inner::filtering::NumericValue,
    filtering::{FilterModifier, ScalarFieldFilter, StringFilter},
    prelude::*,
    sorting::SortDirection,
    tag::search::Filter as TagFilter,
};

#[cfg(feature = "frontend")]
use crate::Pagination;

mod _inner {
    pub(super) use crate::_inner::track::search::*;

    #[cfg(feature = "frontend")]
    pub(super) use crate::_inner::filtering::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    AudioBitrateBps,
    AudioChannelCount,
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

#[cfg(feature = "backend")]
impl From<SortField> for _inner::SortField {
    fn from(from: SortField) -> Self {
        use SortField::*;
        match from {
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            CollectedAt => Self::CollectedAt,
            ContentPath => Self::ContentPath,
            ContentType => Self::ContentType,
            Copyright => Self::Copyright,
            CreatedAt => Self::CreatedAt,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
            Publisher => Self::Publisher,
            RecordedAtDate => Self::RecordedAtDate,
            ReleasedAtDate => Self::ReleasedAtDate,
            ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            TrackNumber => Self::TrackNumber,
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
            AudioBitrateBps => Self::AudioBitrateBps,
            AudioChannelCount => Self::AudioChannelCount,
            AudioDurationMs => Self::AudioDurationMs,
            AudioLoudnessLufs => Self::AudioLoudnessLufs,
            AudioSampleRateHz => Self::AudioSampleRateHz,
            CollectedAt => Self::CollectedAt,
            ContentPath => Self::ContentPath,
            ContentType => Self::ContentType,
            Copyright => Self::Copyright,
            CreatedAt => Self::CreatedAt,
            DiscNumber => Self::DiscNumber,
            DiscTotal => Self::DiscTotal,
            MusicTempoBpm => Self::MusicTempoBpm,
            MusicKeyCode => Self::MusicKeyCode,
            Publisher => Self::Publisher,
            RecordedAtDate => Self::RecordedAtDate,
            ReleasedAtDate => Self::ReleasedAtDate,
            ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            TrackNumber => Self::TrackNumber,
            TrackTotal => Self::TrackTotal,
            UpdatedAt => Self::UpdatedAt,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum StringField {
    ContentPath,
    ContentType,
    Copyright,
    Publisher,
}

#[cfg(feature = "backend")]
impl From<StringField> for _inner::StringField {
    fn from(from: StringField) -> Self {
        use StringField::*;
        match from {
            ContentPath => Self::ContentPath,
            ContentType => Self::ContentType,
            Copyright => Self::Copyright,
            Publisher => Self::Publisher,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::StringField> for StringField {
    fn from(from: _inner::StringField) -> Self {
        use _inner::StringField::*;
        match from {
            ContentPath => Self::ContentPath,
            ContentType => Self::ContentType,
            Copyright => Self::Copyright,
            Publisher => Self::Publisher,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
    RecordedAtDate,
    ReleasedAtDate,
    ReleasedOrigAtDate,
    MusicTempoBpm,
    MusicKeyCode,
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
            RecordedAtDate => Self::RecordedAtDate,
            ReleasedAtDate => Self::ReleasedAtDate,
            ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            TrackNumber => Self::TrackNumber,
            TrackTotal => Self::TrackTotal,
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
            RecordedAtDate => Self::RecordedAtDate,
            ReleasedAtDate => Self::ReleasedAtDate,
            ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            TrackNumber => Self::TrackNumber,
            TrackTotal => Self::TrackTotal,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum DateTimeField {
    CollectedAt,
    RecordedAt,
    ReleasedAt,
    ReleasedOrigAt,
}

#[cfg(feature = "backend")]
impl From<DateTimeField> for _inner::DateTimeField {
    fn from(from: DateTimeField) -> Self {
        use DateTimeField::*;
        match from {
            CollectedAt => Self::CollectedAt,
            RecordedAt => Self::RecordedAt,
            ReleasedAt => Self::ReleasedAt,
            ReleasedOrigAt => Self::ReleasedOrigAt,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::DateTimeField> for DateTimeField {
    fn from(from: _inner::DateTimeField) -> Self {
        use _inner::DateTimeField::*;
        match from {
            CollectedAt => Self::CollectedAt,
            RecordedAt => Self::RecordedAt,
            ReleasedAt => Self::ReleasedAt,
            ReleasedOrigAt => Self::ReleasedOrigAt,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub enum Scope {
    Track,
    Album,
}

#[cfg(feature = "backend")]
impl From<Scope> for _inner::Scope {
    fn from(from: Scope) -> Self {
        match from {
            Scope::Track => Self::Track,
            Scope::Album => Self::Album,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::Scope> for Scope {
    fn from(from: _inner::Scope) -> Self {
        match from {
            _inner::Scope::Track => Self::Track,
            _inner::Scope::Album => Self::Album,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ActorPhraseFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<ActorRole>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub kinds: Vec<ActorKind>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub name_terms: Vec<String>,
}

#[cfg(feature = "backend")]
impl From<ActorPhraseFilter> for _inner::ActorPhraseFilter {
    fn from(from: ActorPhraseFilter) -> Self {
        let ActorPhraseFilter {
            modifier,
            scope,
            roles,
            kinds,
            name_terms,
        } = from;
        Self {
            modifier: modifier.map(Into::into),
            scope: scope.map(Into::into),
            roles: roles.into_iter().map(Into::into).collect(),
            kinds: kinds.into_iter().map(Into::into).collect(),
            name_terms: name_terms.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::ActorPhraseFilter> for ActorPhraseFilter {
    fn from(from: _inner::ActorPhraseFilter) -> Self {
        let _inner::ActorPhraseFilter {
            modifier,
            scope,
            roles,
            kinds,
            name_terms,
        } = from;
        Self {
            modifier: modifier.map(Into::into),
            scope: scope.map(Into::into),
            roles: roles.into_iter().map(Into::into).collect(),
            kinds: kinds.into_iter().map(Into::into).collect(),
            name_terms: name_terms.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct TitlePhraseFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<Scope>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub kinds: Vec<TitleKind>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub name_terms: Vec<String>,
}

#[cfg(feature = "backend")]
impl From<TitlePhraseFilter> for _inner::TitlePhraseFilter {
    fn from(from: TitlePhraseFilter) -> Self {
        let TitlePhraseFilter {
            modifier,
            scope,
            kinds,
            name_terms,
        } = from;
        Self {
            modifier: modifier.map(Into::into),
            scope: scope.map(Into::into),
            kinds: kinds.into_iter().map(Into::into).collect(),
            name_terms: name_terms.into_iter().map(Into::into).collect(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::TitlePhraseFilter> for TitlePhraseFilter {
    fn from(from: _inner::TitlePhraseFilter) -> Self {
        let _inner::TitlePhraseFilter {
            modifier,
            scope,
            kinds,
            name_terms,
        } = from;
        Self {
            modifier: modifier.map(Into::into),
            scope: scope.map(Into::into),
            kinds: kinds.into_iter().map(Into::into).collect(),
            name_terms: name_terms.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub enum Filter {
    Phrase(PhraseFieldFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Condition(ConditionFilter),
    Tag(TagFilter),
    CueLabel(StringFilter),
    AnyTrackUid(Vec<EntityUid>),
    AnyPlaylistUid(Vec<EntityUid>),
    ActorPhrase(ActorPhraseFilter),
    TitlePhrase(TitlePhraseFilter),
    All(Vec<Filter>),
    Any(Vec<Filter>),
    Not(Box<Filter>),
}

#[cfg(feature = "backend")]
impl From<Filter> for _inner::Filter {
    fn from(from: Filter) -> Self {
        use Filter::*;
        match from {
            Phrase(from) => Self::Phrase(from.into()),
            Numeric(from) => Self::Numeric(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
            Condition(from) => Self::Condition(from.into()),
            Tag(from) => Self::Tag(from.into()),
            CueLabel(from) => Self::CueLabel(from.into()),
            AnyTrackUid(from) => {
                Self::AnyTrackUid(from.into_iter().map(EntityUidTyped::from_untyped).collect())
            }
            AnyPlaylistUid(from) => {
                Self::AnyPlaylistUid(from.into_iter().map(EntityUidTyped::from_untyped).collect())
            }
            ActorPhrase(from) => Self::ActorPhrase(from.into()),
            TitlePhrase(from) => Self::TitlePhrase(from.into()),
            All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::Filter> for Filter {
    fn from(from: _inner::Filter) -> Self {
        use _inner::Filter::*;
        match from {
            Phrase(from) => Self::Phrase(from.into()),
            Numeric(from) => Self::Numeric(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
            Condition(from) => Self::Condition(from.into()),
            Tag(from) => Self::Tag(from.into()),
            CueLabel(from) => Self::CueLabel(from.into()),
            AnyTrackUid(from) => Self::AnyTrackUid(from.into_iter().map(Into::into).collect()),
            AnyPlaylistUid(from) => {
                Self::AnyPlaylistUid(from.into_iter().map(Into::into).collect())
            }
            ActorPhrase(from) => Self::ActorPhrase(from.into()),
            TitlePhrase(from) => Self::TitlePhrase(from.into()),
            All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_url_from_content_path: Option<bool>,

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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<SortOrder>,
}

#[cfg(feature = "frontend")]
pub fn client_query_params(
    resolve_url_from_content_path: Option<aoide_core_api::media::source::ResolveUrlFromContentPath>,
    pagination: impl Into<Pagination>,
) -> QueryParams {
    use aoide_core_api::media::source::ResolveUrlFromContentPath;

    let Pagination { limit, offset } = pagination.into();
    let (resolve_url_from_content_path, override_root_url) =
        if let Some(resolve_url_from_content_path) = resolve_url_from_content_path {
            let override_root_url = match resolve_url_from_content_path {
                ResolveUrlFromContentPath::CanonicalRootUrl => None,
                ResolveUrlFromContentPath::OverrideRootUrl { root_url } => Some(root_url),
            };
            (true, override_root_url)
        } else {
            (false, None)
        };
    QueryParams {
        resolve_url_from_content_path: Some(resolve_url_from_content_path),
        override_root_url: override_root_url.map(Into::into),
        limit,
        offset,
    }
}

#[cfg(feature = "frontend")]
pub fn client_request_params(
    params: _inner::Params,
    pagination: impl Into<Pagination>,
) -> (QueryParams, SearchParams) {
    let _inner::Params {
        resolve_url_from_content_path,
        filter,
        ordering,
    } = params;
    let query_params = client_query_params(resolve_url_from_content_path, pagination);
    let search_params = SearchParams {
        filter: filter.map(Into::into),
        ordering: ordering.into_iter().map(Into::into).collect(),
    };
    (query_params, search_params)
}
