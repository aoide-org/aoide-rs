// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(feature = "backend")]
use serde::Deserialize;
#[cfg(feature = "frontend")]
use serde::Serialize;

use aoide_core::tag::FacetId;
use aoide_core_api::{PaginationLimit, PaginationOffset, filter::NumericValue};
use aoide_core_json::{
    entity::EntityUid,
    track::{
        actor::{Kind as ActorKind, Role as ActorRole},
        title::Kind as TitleKind,
    },
    util::clock::DateTime,
};
use url::Url;

#[cfg(feature = "frontend")]
use crate::Pagination;
use crate::{
    SortDirection,
    filter::{FilterModifier, ScalarFieldFilter, StringFilter},
    tag::search::Filter as TagFilter,
};

mod _inner {
    #[cfg(feature = "frontend")]
    pub(super) use crate::_inner::filter::*;
    pub(super) use crate::_inner::track::search::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackSortField")
)]
#[serde(rename_all = "camelCase")]
pub enum SortField {
    AlbumArtist,
    AlbumTitle,
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
    TrackArtist,
    TrackNumber,
    TrackTitle,
    TrackTotal,
    UpdatedAt,
}

#[cfg(feature = "backend")]
impl From<SortField> for _inner::SortField {
    fn from(from: SortField) -> Self {
        use SortField as From;
        match from {
            From::AlbumArtist => Self::AlbumArtist,
            From::AlbumTitle => Self::AlbumTitle,
            From::AudioBitrateBps => Self::AudioBitrateBps,
            From::AudioChannelCount => Self::AudioChannelCount,
            From::AudioChannelMask => Self::AudioChannelMask,
            From::AudioDurationMs => Self::AudioDurationMs,
            From::AudioLoudnessLufs => Self::AudioLoudnessLufs,
            From::AudioSampleRateHz => Self::AudioSampleRateHz,
            From::CollectedAt => Self::CollectedAt,
            From::ContentPath => Self::ContentPath,
            From::ContentType => Self::ContentType,
            From::Copyright => Self::Copyright,
            From::CreatedAt => Self::CreatedAt,
            From::DiscNumber => Self::DiscNumber,
            From::DiscTotal => Self::DiscTotal,
            From::MusicTempoBpm => Self::MusicTempoBpm,
            From::MusicKeyCode => Self::MusicKeyCode,
            From::Publisher => Self::Publisher,
            From::RecordedAtDate => Self::RecordedAtDate,
            From::ReleasedAtDate => Self::ReleasedAtDate,
            From::ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            From::TrackArtist => Self::TrackArtist,
            From::TrackNumber => Self::TrackNumber,
            From::TrackTitle => Self::TrackTitle,
            From::TrackTotal => Self::TrackTotal,
            From::UpdatedAt => Self::UpdatedAt,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::SortField> for SortField {
    fn from(from: _inner::SortField) -> Self {
        use _inner::SortField as From;
        match from {
            From::AlbumArtist => Self::AlbumArtist,
            From::AlbumTitle => Self::AlbumTitle,
            From::AudioBitrateBps => Self::AudioBitrateBps,
            From::AudioChannelCount => Self::AudioChannelCount,
            From::AudioChannelMask => Self::AudioChannelMask,
            From::AudioDurationMs => Self::AudioDurationMs,
            From::AudioLoudnessLufs => Self::AudioLoudnessLufs,
            From::AudioSampleRateHz => Self::AudioSampleRateHz,
            From::CollectedAt => Self::CollectedAt,
            From::ContentPath => Self::ContentPath,
            From::ContentType => Self::ContentType,
            From::Copyright => Self::Copyright,
            From::CreatedAt => Self::CreatedAt,
            From::DiscNumber => Self::DiscNumber,
            From::DiscTotal => Self::DiscTotal,
            From::MusicTempoBpm => Self::MusicTempoBpm,
            From::MusicKeyCode => Self::MusicKeyCode,
            From::Publisher => Self::Publisher,
            From::RecordedAtDate => Self::RecordedAtDate,
            From::ReleasedAtDate => Self::ReleasedAtDate,
            From::ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            From::TrackArtist => Self::TrackArtist,
            From::TrackNumber => Self::TrackNumber,
            From::TrackTitle => Self::TrackTitle,
            From::TrackTotal => Self::TrackTotal,
            From::UpdatedAt => Self::UpdatedAt,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackSortOrder")
)]
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

/// Selectable fields, including
///
/// * `contentType` - RFC 6838 media type
/// * `contentPath`
/// * ...
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
        use StringField as From;
        match from {
            From::ContentPath => Self::ContentPath,
            From::ContentType => Self::ContentType,
            From::Copyright => Self::Copyright,
            From::Publisher => Self::Publisher,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::StringField> for StringField {
    fn from(from: _inner::StringField) -> Self {
        use _inner::StringField as From;
        match from {
            From::ContentPath => Self::ContentPath,
            From::ContentType => Self::ContentType,
            From::Copyright => Self::Copyright,
            From::Publisher => Self::Publisher,
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
    ArtworkDataSize,
    ArtworkImageHeight,
    ArtworkImageWidth,
    AudioBitrateBps,
    AudioChannelCount,
    AudioChannelMask,
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
        use NumericField as From;
        match from {
            From::AdvisoryRating => Self::AdvisoryRating,
            From::ArtworkDataSize => Self::ArtworkDataSize,
            From::ArtworkImageHeight => Self::ArtworkImageHeight,
            From::ArtworkImageWidth => Self::ArtworkImageWidth,
            From::AudioBitrateBps => Self::AudioBitrateBps,
            From::AudioChannelCount => Self::AudioChannelCount,
            From::AudioChannelMask => Self::AudioChannelMask,
            From::AudioDurationMs => Self::AudioDurationMs,
            From::AudioSampleRateHz => Self::AudioSampleRateHz,
            From::AudioLoudnessLufs => Self::AudioLoudnessLufs,
            From::DiscNumber => Self::DiscNumber,
            From::DiscTotal => Self::DiscTotal,
            From::MusicTempoBpm => Self::MusicTempoBpm,
            From::MusicKeyCode => Self::MusicKeyCode,
            From::RecordedAtDate => Self::RecordedAtDate,
            From::ReleasedAtDate => Self::ReleasedAtDate,
            From::ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            From::TrackNumber => Self::TrackNumber,
            From::TrackTotal => Self::TrackTotal,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::NumericField> for NumericField {
    fn from(from: _inner::NumericField) -> Self {
        use _inner::NumericField as From;
        match from {
            From::AdvisoryRating => Self::AdvisoryRating,
            From::ArtworkDataSize => Self::ArtworkDataSize,
            From::ArtworkImageHeight => Self::ArtworkImageHeight,
            From::ArtworkImageWidth => Self::ArtworkImageWidth,
            From::AudioBitrateBps => Self::AudioBitrateBps,
            From::AudioChannelCount => Self::AudioChannelCount,
            From::AudioChannelMask => Self::AudioChannelMask,
            From::AudioDurationMs => Self::AudioDurationMs,
            From::AudioSampleRateHz => Self::AudioSampleRateHz,
            From::AudioLoudnessLufs => Self::AudioLoudnessLufs,
            From::DiscNumber => Self::DiscNumber,
            From::DiscTotal => Self::DiscTotal,
            From::MusicTempoBpm => Self::MusicTempoBpm,
            From::MusicKeyCode => Self::MusicKeyCode,
            From::RecordedAtDate => Self::RecordedAtDate,
            From::ReleasedAtDate => Self::ReleasedAtDate,
            From::ReleasedOrigAtDate => Self::ReleasedOrigAtDate,
            From::TrackNumber => Self::TrackNumber,
            From::TrackTotal => Self::TrackTotal,
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
        use DateTimeField as From;
        match from {
            From::CollectedAt => Self::CollectedAt,
            From::RecordedAt => Self::RecordedAt,
            From::ReleasedAt => Self::ReleasedAt,
            From::ReleasedOrigAt => Self::ReleasedOrigAt,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::DateTimeField> for DateTimeField {
    fn from(from: _inner::DateTimeField) -> Self {
        use _inner::DateTimeField as From;
        match from {
            From::CollectedAt => Self::CollectedAt,
            From::RecordedAt => Self::RecordedAt,
            From::ReleasedAt => Self::ReleasedAt,
            From::ReleasedOrigAt => Self::ReleasedOrigAt,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackSearchConditionFilter")
)]
#[serde(rename_all = "camelCase")]
pub enum ConditionFilter {
    SourceTracked,
    SourceUntracked,
}

#[cfg(feature = "backend")]
impl From<ConditionFilter> for _inner::ConditionFilter {
    fn from(from: ConditionFilter) -> Self {
        use ConditionFilter as From;
        match from {
            From::SourceTracked => Self::SourceTracked,
            From::SourceUntracked => Self::SourceUntracked,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::ConditionFilter> for ConditionFilter {
    fn from(from: _inner::ConditionFilter) -> Self {
        use _inner::ConditionFilter as From;
        match from {
            From::SourceTracked => Self::SourceTracked,
            From::SourceUntracked => Self::SourceUntracked,
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

/// Matches a sequence of terms on any of the given string fields. A match for at least one of the
/// fields is sufficient.
///
/// The array consists of two elements:
///
/// 1. The fields that are matched against the terms. If no fields are specified all available
///    string fields are matched.
/// 2. The terms are concatenated with wildcards, and filtered using case-insensitive "contains"
///    semantics for each of the selected fields, e.g. `["pa", "lab", "ll"]` or `["tt", "ll"]`
///    both match the string "Patti LaBelle". An empty array matches both empty and missing/null
///    fields.
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

/// The scope of a field or property in a track.
#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackScope")
)]
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

    /// A non-empty array will match any of the given roles. An empty or missing array will match any role.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub roles: Vec<ActorRole>,

    /// A non-empty array will match any of the given kinds. An empty or missing array will match any kind.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub kinds: Vec<ActorKind>,

    /// The terms are concatenated with wildcards, and filtered using case-insensitive "contains"
    /// semantics for each of the selected fields, e.g. `["pa", "lab", "ll"]` or `["tt", "ll"]` both
    /// match the string "Patti LaBelle".
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
            name_terms,
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
            name_terms,
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

    /// A non-empty array will match any of the given kinds. An empty or missing array will match any kind.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub kinds: Vec<TitleKind>,

    /// The terms are concatenated with wildcards, and filtered using case-insensitive "contains"
    /// semantics for each of the selected fields, e.g. `["pa", "lab", "ll"]` or `["tt", "ll"]` both
    /// match the string "Patti LaBelle".
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
            name_terms,
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
            name_terms,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackSearchFilter")
)]
#[serde(rename_all = "camelCase")]
pub enum Filter {
    Phrase(PhraseFieldFilter),
    ActorPhrase(ActorPhraseFilter),
    TitlePhrase(TitlePhraseFilter),
    Numeric(NumericFieldFilter),
    DateTime(DateTimeFieldFilter),
    Condition(ConditionFilter),
    Tag(TagFilter),
    CueLabel(StringFilter),
    AnyTrackUid(Vec<EntityUid>),
    AnyPlaylistUid(Vec<EntityUid>),
    All(Vec<Filter>),
    Any(Vec<Filter>),
    Not(Box<Filter>),
}

#[cfg(feature = "backend")]
impl From<Filter> for _inner::Filter {
    fn from(from: Filter) -> Self {
        use Filter as From;
        use aoide_core::EntityUidTyped;
        match from {
            From::Phrase(from) => Self::Phrase(from.into()),
            From::ActorPhrase(from) => Self::ActorPhrase(from.into()),
            From::TitlePhrase(from) => Self::TitlePhrase(from.into()),
            From::Numeric(from) => Self::Numeric(from.into()),
            From::DateTime(from) => Self::DateTime(from.into()),
            From::Condition(from) => Self::Condition(from.into()),
            From::Tag(from) => Self::Tag(from.into()),
            From::CueLabel(from) => Self::CueLabel(from.into()),
            From::AnyTrackUid(from) => {
                Self::AnyTrackUid(from.into_iter().map(EntityUidTyped::from_untyped).collect())
            }
            From::AnyPlaylistUid(from) => {
                Self::AnyPlaylistUid(from.into_iter().map(EntityUidTyped::from_untyped).collect())
            }
            From::All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            From::Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            From::Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_inner::Filter> for Filter {
    fn from(from: _inner::Filter) -> Self {
        use _inner::Filter as From;
        match from {
            From::Phrase(from) => Self::Phrase(from.into()),
            From::ActorPhrase(from) => Self::ActorPhrase(from.into()),
            From::TitlePhrase(from) => Self::TitlePhrase(from.into()),
            From::Numeric(from) => Self::Numeric(from.into()),
            From::DateTime(from) => Self::DateTime(from.into()),
            From::Condition(from) => Self::Condition(from.into()),
            From::Tag(from) => Self::Tag(from.into()),
            From::CueLabel(from) => Self::CueLabel(from.into()),
            From::AnyTrackUid(from) => {
                Self::AnyTrackUid(from.into_iter().map(Into::into).collect())
            }
            From::AnyPlaylistUid(from) => {
                Self::AnyPlaylistUid(from.into_iter().map(Into::into).collect())
            }
            From::All(from) => Self::All(from.into_iter().map(Into::into).collect()),
            From::Any(from) => Self::Any(from.into_iter().map(Into::into).collect()),
            From::Not(from) => Self::Not(Box::new((*from).into())),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackSearchQueryParams")
)]
pub struct QueryParams {
    /// Resolve the actual URL of the media source from the stored path according
    /// to the collection's media source configuration.
    ///
    /// Only supported for local files with virtual paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolve_url_from_content_path: Option<bool>,

    /// Resolve the actual URL of the media source from the stored path according
    /// to the given root URL. The provided URL overrides the root URL from the
    /// collection's media source configuration for this particular request.
    ///
    /// Only supported for local files with virtual paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_root_url: Option<Url>,

    /// Encode all custom tags that are not supported as native file tags
    /// into the tags with the given facet, typically either "cgrp"
    /// (content group/grouping) or "comm" (comment).
    ///
    /// This option is useful for interoperability with applications that
    /// only support the common file tags and for file synchronization.
    /// Custom tags are not encoded if omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encode_gigtags: Option<FacetId>,

    /// Maximum number of items to return or unlimited if missing.
    ///
    /// When this parameter is specified its value is evaluated and
    /// applied, i.e. a value of 0 will result in an empty set of
    /// returned items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,

    /// Number of items to skip before returning the results or 0 if missing.
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
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackSearchParams")
)]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub order: Vec<SortOrder>,
}

#[cfg(feature = "frontend")]
pub fn client_query_params(
    resolve_url_from_content_path: Option<aoide_core_api::media::source::ResolveUrlFromContentPath>,
    encode_gigtags: Option<aoide_core::tag::FacetId>,
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
        encode_gigtags,
        limit,
        offset,
    }
}

#[cfg(feature = "frontend")]
pub fn client_request_params(
    params: _inner::Params,
    encode_gigtags: Option<aoide_core::tag::FacetId>,
    pagination: impl Into<Pagination>,
) -> (QueryParams, SearchParams) {
    let _inner::Params {
        resolve_url_from_content_path,
        filter,
        order,
    } = params;
    let query_params =
        client_query_params(resolve_url_from_content_path, encode_gigtags, pagination);
    let search_params = SearchParams {
        filter: filter.map(Into::into),
        order: order.into_iter().map(Into::into).collect(),
    };
    (query_params, search_params)
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "backend")]
    #[test]
    fn deserialize_filter_examples() {
        for filter_name in ["any_date_in_1990s", "missing_genre_tag"] {
            assert!(
                serde_json::from_reader::<_, super::Filter>(std::io::BufReader::new(
                    std::fs::File::open(format!(
                        "../../examples/json/track-search/filter_{filter_name}.json"
                    ))
                    .unwrap()
                ))
                .is_ok()
            );
        }
    }
}
