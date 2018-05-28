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

use aoide_core::domain::metadata::Score;
use aoide_core::domain::track::TrackBody;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SortField {
    pub dir: SortDirection,

    pub field: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum FilterModifier {
    Inverse,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFilterParams {
    pub value: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum StringFilter {
    StartsWith(StringFilterParams), // head
    EndsWith(StringFilterParams),   // tail
    Contains(StringFilterParams),   // part
    Matches(StringFilterParams),    // all
}

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScoreFilterParams {
    pub value: Score,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum ScoreFilter {
    LessThan(ScoreFilterParams),
    GreaterThan(ScoreFilterParams),
    EqualTo(ScoreFilterParams),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LocateParams {
    #[serde(rename = "uri")]
    pub uri_filter: StringFilter,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum ReplaceMode {
    UpdateOnly,
    UpdateOrCreate,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ReplaceParams {
    #[serde(skip_serializing_if = "String::is_empty", default)]
    pub uri: String,

    pub mode: ReplaceMode,

    pub body: TrackBody,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TagFilter {
    // Facets are only matched with equals. Use an empty string
    // for matching tags without a facet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facet: Option<String>,

    #[serde(rename = "term", skip_serializing_if = "Option::is_none")]
    pub term_filter: Option<StringFilter>,

    #[serde(rename = "score", skip_serializing_if = "Option::is_none")]
    pub score_filter: Option<ScoreFilter>,
}

impl TagFilter {
    pub fn any_facet() -> Option<String> {
        None
    }

    pub fn no_facet() -> Option<String> {
        Some(String::default())
    }

    pub fn any_term() -> Option<StringFilter> {
        None
    }

    pub fn any_score() -> Option<ScoreFilter> {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum PhraseFilterField {
    Source, // percent-decoded URI
    Grouping,
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
    // and filtered using "contains" semantics with selected
    // (or all) fields
    pub phrase: String,

    // Empty == All
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub fields: Vec<PhraseFilterField>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub modifier: Option<FilterModifier>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SearchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phrase_filter: Option<PhraseFilter>,

    // 1st level: Conjunction
    // 2nd level: Disjunction
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tag_filters: Vec<Vec<TagFilter>>,
    // TODO: Implement sorting
    //#[serde(skip_serializing_if = "Vec::is_empty", default)]
    //pub sort_fields: Vec<SortField>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum FrequencyField {
    Grouping,
    TrackTitle,
    AlbumTitle,
    TrackArtist,
    AlbumArtist,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFrequency {
    pub value: Option<String>,
    pub count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StringFrequencyResult {
    pub field: FrequencyField,
    pub frequencies: Vec<StringFrequency>,
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_tag_filter() {
        assert_eq!(TagFilter::any_facet(), TagFilter::default().facet);
        assert_ne!(TagFilter::no_facet(), TagFilter::default().facet);
        assert_eq!(TagFilter::any_term(), TagFilter::default().term_filter);
        assert_eq!(TagFilter::any_score(), TagFilter::default().score_filter);
    }
}
