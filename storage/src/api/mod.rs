// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub mod collection;
pub mod entity;
pub mod serde;
pub mod track;

pub type PaginationOffset = u64;

pub type PaginationLimit = u64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
pub enum FilterModifier {
    Complement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum StringCompare {
    StartsWith, // head (case-insensitive)
    EndsWith,   // tail (case-insensitive)
    Contains,   // part (case-insensitive)
    Matches,    // all (case-insensitive)
    Equals,     // all (case-sensitive)
}

/// Predicates for matching strings
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum StringPredicate {
    // Case-sensitive comparison
    StartsWith(String),
    StartsNotWith(String),
    EndsWith(String),
    EndsNotWith(String),
    Contains(String),
    ContainsNot(String),
    Matches(String),
    MatchesNot(String),
    // Case-sensitive comparison
    Equals(String),
    EqualsNot(String),
}

impl<'a> From<&'a StringPredicate> for (StringCompare, &'a String, bool) {
    fn from(from: &'a StringPredicate) -> (StringCompare, &'a String, bool) {
        match from {
            StringPredicate::StartsWith(s) => (StringCompare::StartsWith, s, true),
            StringPredicate::StartsNotWith(s) => (StringCompare::StartsWith, s, false),
            StringPredicate::EndsWith(s) => (StringCompare::EndsWith, s, true),
            StringPredicate::EndsNotWith(s) => (StringCompare::EndsWith, s, false),
            StringPredicate::Contains(s) => (StringCompare::Contains, s, true),
            StringPredicate::ContainsNot(s) => (StringCompare::Contains, s, false),
            StringPredicate::Matches(s) => (StringCompare::Matches, s, true),
            StringPredicate::MatchesNot(s) => (StringCompare::Matches, s, false),
            StringPredicate::Equals(s) => (StringCompare::Equals, s, true),
            StringPredicate::EqualsNot(s) => (StringCompare::Equals, s, false),
        }
    }
}

/// Predicates for matching URI strings (case-sensitive)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum UriPredicate {
    Prefix(String),
    Exact(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct UriRelocation {
    pub predicate: UriPredicate,
    pub replacement: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    // Facets are always matched with equals. Use an empty vector
    // for matching only tags without a facet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringPredicate>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<NumericPredicate>,
}

impl TagFilter {
    pub fn any_facet() -> Option<Vec<String>> {
        None
    }

    pub fn no_facet() -> Option<Vec<String>> {
        Some(Vec::default())
    }

    pub fn any_term() -> Option<StringPredicate> {
        None
    }

    pub fn any_score() -> Option<NumericPredicate> {
        None
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MarkerFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<StringPredicate>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum StringField {
    SourceUri, // percent-decoded URI
    ContentType,
    TrackTitle,
    TrackArtist,
    TrackComposer,
    AlbumTitle,
    AlbumArtist,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum NumericField {
    AudioBitRate,
    AudioChannelCount,
    AudioDuration,
    AudioSampleRate,
    AudioLoudness,
    TrackIndex,
    TrackCount,
    ReleaseYear,
    MusicTempo,
    MusicKey,
}

pub type NumericValue = f64;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum NumericPredicate {
    #[serde(rename = "lt")]
    LessThan(NumericValue),
    #[serde(rename = "le")]
    LessOrEqual(NumericValue),
    #[serde(rename = "gt")]
    GreaterThan(NumericValue),
    #[serde(rename = "ge")]
    GreaterOrEqual(NumericValue),
    #[serde(rename = "eq")]
    Equal(Option<NumericValue>),
    #[serde(rename = "ne")]
    NotEqual(Option<NumericValue>),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct NumericFilter {
    pub field: NumericField,

    pub value: NumericPredicate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PhraseFilter {
    // Empty == All available string fields are considered
    // Disjunction, i.e. a match in one of the fields is sufficient
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<StringField>,

    // Concatenated with wildcards and filtered using
    // case-insensitive "contains" semantics against each
    // of the selected fields, e.g. ["pa", "la", "bell"]
    // ["tt, ll"] will both match "Patti LaBelle". An empty
    // argument matches empty as well as missing/null fields.
    pub terms: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LocateTracksParams {
    pub uri: StringPredicate,
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
    // The URI for looking up the existing track (if any)
    // that gets replaced.
    pub uri: String,

    pub track: Track,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplaceTracksParams {
    pub mode: ReplaceMode,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub replacements: Vec<TrackReplacement>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplacedTracks {
    pub created: Vec<EntityHeader>,
    pub updated: Vec<EntityHeader>,
    pub skipped: Vec<EntityHeader>,
    pub rejected: Vec<String>,  // e.g. ambiguous or inconsistent
    pub discarded: Vec<String>, // e.g. nonexistent and need to be created
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum TrackSortField {
    InCollectionSince, // = recently added (only if searching in a single collection)
    LastRevisionedAt,  // = recently modified (created or updated)
    TrackTitle,
    TrackArtist,
    TrackIndex,
    TrackCount,
    AlbumTitle,
    AlbumArtist,
    ReleaseYear,
    MusicTempo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum TagSortField {
    Facet,
    Label,
    Score,
    Count,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "dsc")]
    Descending,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSortOrder {
    pub field: TrackSortField,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SortDirection>,
}

impl TrackSortOrder {
    pub fn default_direction(field: TrackSortField) -> SortDirection {
        match field {
            TrackSortField::InCollectionSince | TrackSortField::LastRevisionedAt => {
                SortDirection::Descending
            }
            _ => SortDirection::Ascending,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagSortOrder {
    pub field: TagSortField,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SortDirection>,
}

impl TagSortOrder {
    pub fn default_direction(field: TagSortField) -> SortDirection {
        match field {
            TagSortField::Score | TagSortField::Count => SortDirection::Descending,
            _ => SortDirection::Ascending,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum TrackSearchFilter {
    Phrase(PhraseFilter),
    Numeric(NumericFilter),
    Tag(TagFilter),
    Marker(MarkerFilter),
    All(Vec<TrackSearchFilter>),
    Any(Vec<TrackSearchFilter>),
    Not(Box<TrackSearchFilter>),
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchTracksParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<TrackSearchFilter>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TrackSortOrder>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CountTracksByAlbumParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_release_year: Option<i16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_release_year: Option<i16>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TrackSortOrder>,
}

// TODO: Replace with #[serde(default_expr = "true")] if available
// See also: https://github.com/serde-rs/serde/pull/1490
fn expr_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CountTracksByTagParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Facet>>,

    #[serde(skip_serializing_if = "std::ops::Not::not", default = "expr_true")]
    pub include_non_faceted_tags: bool,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TagSortOrder>,
}

impl CountTracksByTagParams {
    pub fn dedup_facets(self) -> Self {
        let facets = self.facets.map(|mut facets| {
            facets.sort();
            facets.dedup();
            facets
        });
        Self { facets, ..self }
    }
}

impl Default for CountTracksByTagParams {
    fn default() -> Self {
        Self {
            facets: Default::default(),
            include_non_faceted_tags: true,
            ordering: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CountTracksByTagFacetParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets: Option<Vec<Facet>>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ordering: Vec<TagSortOrder>,
}

impl CountTracksByTagFacetParams {
    pub fn dedup_facets(self) -> Self {
        let facets = self.facets.map(|mut facets| {
            facets.sort();
            facets.dedup();
            facets
        });
        Self { facets, ..self }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringCount {
    #[serde(rename = "val")]
    pub value: Option<String>,

    #[serde(rename = "cnt")]
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldStrings(StringField, Vec<StringCount>);

impl FieldStrings {
    pub const fn new(field: StringField, counts: Vec<StringCount>) -> Self {
        FieldStrings(field, counts)
    }

    pub fn field(&self) -> &StringField {
        &self.0
    }

    pub fn counts(&self) -> &Vec<StringCount> {
        &self.1
    }
}

#[derive(Clone, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFacetCount(Facet, usize);

impl TagFacetCount {
    pub const fn new(facet: Facet, count: usize) -> Self {
        Self(facet, count)
    }

    pub fn facet(&self) -> &Facet {
        &self.0
    }

    pub fn count(&self) -> usize {
        self.1
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagCount {
    #[serde(rename = "fct", skip_serializing_if = "Option::is_none")]
    pub facet: Option<Facet>,

    #[serde(rename = "lbl", skip_serializing_if = "Option::is_none")]
    pub label: Option<Label>,

    #[serde(rename = "avg")]
    pub avg_score: Score,

    #[serde(rename = "cnt")]
    pub count: usize,
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
