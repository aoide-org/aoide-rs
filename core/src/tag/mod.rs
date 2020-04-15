// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

use super::*;

use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

///////////////////////////////////////////////////////////////////////
// Score
///////////////////////////////////////////////////////////////////////

pub type ScoreValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Score(ScoreValue);

impl Score {
    pub const fn min_value() -> ScoreValue {
        0.0
    }

    pub const fn max_value() -> ScoreValue {
        1.0
    }

    pub fn clamp_value(value: ScoreValue) -> ScoreValue {
        //value.clamp(Self::min(), Self::max())
        value.min(Self::max_value()).max(Self::min_value())
    }

    pub const fn min() -> Self {
        Self(Self::min_value())
    }

    pub const fn max() -> Self {
        Self(Self::max_value())
    }

    pub const fn from_inner(inner: ScoreValue) -> Self {
        Self(inner)
    }

    pub const fn into_inner(self) -> ScoreValue {
        self.0
    }

    // Convert to percentage value with a single decimal digit
    pub fn to_percentage(self) -> ScoreValue {
        debug_assert!(self.validate().is_ok());
        (self.0 * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
    pub fn to_permille(self) -> u16 {
        debug_assert!(self.validate().is_ok());
        (self.0 * ScoreValue::from(1_000)).round() as u16
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ScoreInvalidity {
    OutOfRange,
}

impl Validate for Score {
    type Invalidity = ScoreInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::min() && *self <= Self::max()),
                ScoreInvalidity::OutOfRange,
            )
            .into()
    }
}

impl From<Score> for ScoreValue {
    fn from(from: Score) -> Self {
        from.into_inner()
    }
}

impl From<ScoreValue> for Score {
    fn from(from: ScoreValue) -> Self {
        Self::from_inner(from)
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_assert!(self.validate().is_ok());
        write!(f, "{:.1}%", self.to_percentage())
    }
}

pub trait Scored {
    fn score(&self) -> Score;
}

impl Scored for Score {
    fn score(&self) -> Self {
        *self
    }
}

///////////////////////////////////////////////////////////////////////
// Label
///////////////////////////////////////////////////////////////////////

pub type LabelValue = String;

/// The name of a tag.
///
/// Format: Uniccode string without leading/trailing whitespace
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Label(LabelValue);

impl Label {
    pub fn clamp_value(value: impl Into<LabelValue>) -> LabelValue {
        // TODO: Truncate the given string instead of creating a new string
        value.into().trim().to_string()
    }

    pub const fn from_inner(inner: LabelValue) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> LabelValue {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LabelInvalidity {
    Empty,
    Format,
}

impl Validate for Label {
    type Invalidity = LabelInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0.is_empty(), LabelInvalidity::Empty)
            .invalidate_if(self.0.trim().len() != self.0.len(), LabelInvalidity::Format)
            .into()
    }
}

impl From<LabelValue> for Label {
    fn from(from: LabelValue) -> Self {
        Self::from_inner(from)
    }
}

impl From<Label> for LabelValue {
    fn from(from: Label) -> Self {
        from.into_inner()
    }
}

impl AsRef<LabelValue> for Label {
    fn as_ref(&self) -> &LabelValue {
        &self.0
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for Label {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

pub trait Labeled {
    fn label(&self) -> Option<&Label>;
}

impl Labeled for Label {
    fn label(&self) -> Option<&Self> {
        Some(self)
    }
}

///////////////////////////////////////////////////////////////////////
// Facet
///////////////////////////////////////////////////////////////////////

pub type FacetValue = String;

/// Identifier for a category of tags.
///
/// Format: ASCII string, no uppercase characters, no whitespace
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Facet(FacetValue);

impl Facet {
    pub fn clamp_value(mut value: FacetValue) -> FacetValue {
        value.retain(Self::is_valid_char);
        value
    }

    pub const fn from_inner(inner: FacetValue) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> FacetValue {
        self.0
    }

    fn is_invalid_char(c: char) -> bool {
        !c.is_ascii() || c.is_ascii_whitespace() || c.is_ascii_uppercase()
    }

    fn is_valid_char(c: char) -> bool {
        !Self::is_invalid_char(c)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FacetInvalidity {
    Empty,
    Format,
}

impl Validate for Facet {
    type Invalidity = FacetInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0.is_empty(), FacetInvalidity::Empty)
            .invalidate_if(
                self.0.chars().any(Facet::is_invalid_char),
                FacetInvalidity::Format,
            )
            .into()
    }
}

impl From<FacetValue> for Facet {
    fn from(from: FacetValue) -> Self {
        Self::from_inner(from)
    }
}

impl From<Facet> for FacetValue {
    fn from(from: Facet) -> Self {
        from.into_inner()
    }
}

impl AsRef<FacetValue> for Facet {
    fn as_ref(&self) -> &FacetValue {
        &self.0
    }
}

impl AsRef<str> for Facet {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl FromStr for Facet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.into()))
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub trait Faceted {
    fn facet(&self) -> Option<&Facet>;
}

impl Faceted for Facet {
    fn facet(&self) -> Option<&Self> {
        Some(self)
    }
}

#[derive(Clone, Default, Debug)]
pub struct FacetKey(Option<Facet>);

impl FacetKey {
    pub const fn from_inner(inner: Option<Facet>) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> Option<Facet> {
        self.0
    }
}

impl From<Option<Facet>> for FacetKey {
    fn from(from: Option<Facet>) -> Self {
        FacetKey::from_inner(from)
    }
}

impl From<FacetKey> for Option<Facet> {
    fn from(from: FacetKey) -> Self {
        from.into_inner()
    }
}

impl From<Facet> for FacetKey {
    fn from(from: Facet) -> Self {
        FacetKey(Some(from))
    }
}

impl AsRef<Option<Facet>> for FacetKey {
    fn as_ref(&self) -> &Option<Facet> {
        &self.0
    }
}

impl AsRef<str> for FacetKey {
    fn as_ref(&self) -> &str {
        match &self.0 {
            Some(facet) => facet.as_ref(),
            None => "",
        }
    }
}

impl FromStr for FacetKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s.is_empty() {
            None.into()
        } else {
            Some(Facet::from_inner(s.into())).into()
        })
    }
}

impl fmt::Display for FacetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ref())
    }
}

impl PartialEq for FacetKey {
    fn eq(&self, other: &Self) -> bool {
        let self_str: &str = self.as_ref();
        let other_str: &str = other.as_ref();
        self_str == other_str
    }
}

impl Eq for FacetKey {}

impl Ord for FacetKey {
    fn cmp(&self, other: &Self) -> Ordering {
        let self_str: &str = self.as_ref();
        let other_str: &str = other.as_ref();
        self_str.cmp(other_str)
    }
}

impl PartialOrd for FacetKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for FacetKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let self_str: &str = self.as_ref();
        self_str.hash(state);
    }
}

///////////////////////////////////////////////////////////////////////
// PlainTag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct PlainTag {
    pub label: Option<Label>,
    pub score: Score,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PlainTagInvalidity {
    Label(LabelInvalidity),
    Score(ScoreInvalidity),
}

impl Validate for PlainTag {
    type Invalidity = PlainTagInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.label, PlainTagInvalidity::Label)
            .validate_with(&self.score, PlainTagInvalidity::Score)
            .into()
    }
}

impl PlainTag {
    pub const fn default_score() -> Score {
        Score::max()
    }
}

impl Default for PlainTag {
    fn default() -> Self {
        Self {
            label: None,
            score: Self::default_score(),
        }
    }
}

impl Labeled for PlainTag {
    fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }
}

impl Scored for PlainTag {
    fn score(&self) -> Score {
        self.score
    }
}

///////////////////////////////////////////////////////////////////////
// Tags
///////////////////////////////////////////////////////////////////////

/// Unified map of both plain and faceted tags
pub type TagsMap = HashMap<FacetKey, Vec<PlainTag>>;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Tags(TagsMap);

impl Tags {
    pub const fn from_inner(inner: TagsMap) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> TagsMap {
        self.0
    }
}

impl From<TagsMap> for Tags {
    fn from(from: TagsMap) -> Self {
        Self::from_inner(from)
    }
}

impl From<Tags> for TagsMap {
    fn from(from: Tags) -> Self {
        from.into_inner()
    }
}

impl AsRef<TagsMap> for Tags {
    fn as_ref(&self) -> &TagsMap {
        &self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TagsInvalidity {
    Facet(FacetInvalidity),
    PlainTag(PlainTagInvalidity),
    DuplicateLabels,
}

impl Tags {
    fn validate_iter<'a, 'b, I>(
        context: ValidationContext<TagsInvalidity>,
        tags_iter: I,
    ) -> ValidationResult<TagsInvalidity>
    where
        I: Iterator<Item = (&'a FacetKey, &'b Vec<PlainTag>)>,
    {
        tags_iter
            .fold(context, |mut context, (facet_key, plain_tags)| {
                let facet: &Option<Facet> = facet_key.as_ref();
                context = context.validate_with(facet, TagsInvalidity::Facet);
                for plain_tag in plain_tags {
                    context = context.validate_with(plain_tag, TagsInvalidity::PlainTag);
                }
                let mut unique_labels: Vec<_> = plain_tags.iter().map(|t| &t.label).collect();
                unique_labels.sort_unstable();
                unique_labels.dedup();
                context = context.invalidate_if(
                    unique_labels.len() < plain_tags.len(),
                    TagsInvalidity::DuplicateLabels,
                );
                context
            })
            .into()
    }
}

impl Validate for Tags {
    type Invalidity = TagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        Self::validate_iter(ValidationContext::new(), self.0.iter())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
