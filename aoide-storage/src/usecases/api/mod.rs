// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

#[cfg(test)]
mod tests;

use aoide_core::audio::Duration;
use aoide_core::domain::{entity::EntityHeader, metadata::{Score, ScoredTag}, music::ScoredGenre, track::TrackBody};

pub type PaginationOffset = u64;

pub type PaginationLimit = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Pagination {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,
}

impl Pagination {
    pub fn none() -> Self {
        Pagination {
            offset: None,
            limit: None,
        }
    }

    pub fn is_none(&self) -> bool {
        self == &Self::none()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ConditionModifier {
    Complement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum FilterModifier {
    Inverse,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringConditionParams {
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<ConditionModifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum StringCondition {
    StartsWith(StringConditionParams), // head
    EndsWith(StringConditionParams),   // tail
    Contains(StringConditionParams),   // part
    Matches(StringConditionParams),    // all
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScoreConditionParams {
    pub value: Score,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<ConditionModifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ScoreCondition {
    LessThan(ScoreConditionParams),
    GreaterThan(ScoreConditionParams),
    EqualTo(ScoreConditionParams),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct GenreFilter {
    #[serde(rename = "score", skip_serializing_if = "Option::is_none")]
    pub score_condition: Option<ScoreCondition>,

    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name_condition: Option<StringCondition>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

impl GenreFilter {
    pub fn any_score() -> Option<ScoreCondition> {
        None
    }

    pub fn any_name() -> Option<StringCondition> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFilter {
    #[serde(rename = "score", skip_serializing_if = "Option::is_none")]
    pub score_condition: Option<ScoreCondition>,

    #[serde(rename = "term", skip_serializing_if = "Option::is_none")]
    pub term_condition: Option<StringCondition>,

    // Facets are always matched with equals. Use an empty string
    // for matching tags without a facet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

impl TagFilter {
    pub fn any_facet() -> Option<String> {
        None
    }

    pub fn no_facet() -> Option<String> {
        Some(String::default())
    }

    pub fn any_term() -> Option<StringCondition> {
        None
    }

    pub fn any_score() -> Option<ScoreCondition> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum NumericField {
    ChannelsCount,
    DurationMs,
    SamplerateHz,
    BitrateBps,
    TempoBpm,
    KeysigCode,
    TimesigUpper,
    TimesigLower,
}

pub type NumericValue = f64;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NumericValueConditionParams {
    pub value: NumericValue,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<ConditionModifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum NumericValueCondition {
    LessThan(NumericValueConditionParams),
    GreaterThan(NumericValueConditionParams),
    EqualTo(NumericValueConditionParams),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NumericFilter {
    pub field: NumericField,

    pub condition: NumericValueCondition,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum PhraseField {
    Source, // percent-decoded URI
    MediaType,
    TrackTitle,
    AlbumTitle,
    TrackArtist,
    AlbumArtist,
    Comments, // all comments, i.e. independent of owner
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PhraseFilter {
    // Tokenized by whitespace, concatenized with wildcards,
    // and filtered using "contains" semantics against any
    // of the selected (or all) fields
    pub query: String,

    // Empty == All
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<PhraseField>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LocateParams {
    #[serde(rename = "uri")]
    pub uri_filter: StringCondition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ReplaceMode {
    UpdateOnly,
    UpdateOrCreate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackReplacement {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub uri: String,

    pub track: TrackBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackReplacementParams {
    pub mode: ReplaceMode,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub replacements: Vec<TrackReplacement>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackReplacementReport {
    pub created: Vec<EntityHeader>,
    pub updated: Vec<EntityHeader>,
    pub skipped: Vec<EntityHeader>,
    pub rejected: Vec<String>, // e.g. ambiguous or inconsistent
    pub discarded: Vec<String>, // e.g. nonexistent and need to be created
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum TrackSortField {
    InCollectionSince, // = recently added (only if searching in a single collection)
    LastRevisionedAt,  // = recently modified (created or updated)
    TrackTitle,
    AlbumTitle,
    TrackArtist,
    AlbumArtist,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "desc")]
    Descending,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSort {
    pub field: TrackSortField,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SortDirection>,
}

impl TrackSort {
    pub fn default_direction(field: TrackSortField) -> SortDirection {
        match field {
            TrackSortField::InCollectionSince | TrackSortField::LastRevisionedAt => {
                SortDirection::Descending
            }
            _ => SortDirection::Ascending,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phrase_filter: Option<PhraseFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub genre_filters: Vec<GenreFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tag_filters: Vec<TagFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub numeric_filters: Vec<NumericFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TrackSort>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum CountableStringField {
    MediaType,
    TrackTitle,
    AlbumTitle,
    TrackArtist,
    AlbumArtist,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringCount {
    pub value: Option<String>,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFieldCounts {
    pub field: CountableStringField,
    pub counts: Vec<StringCount>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MediaTypeStats {
    pub media_type: String,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResourceStats {
    pub count: usize,
    pub duration: Duration,
    pub media_types: Vec<MediaTypeStats>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScoredGenreCount {
    pub genre: ScoredGenre,

    pub count: usize,
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFacetCount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>,

    pub count: usize,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScoredTagCount {
    pub tag: ScoredTag,

    pub count: usize,
}
