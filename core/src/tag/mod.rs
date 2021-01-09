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

///////////////////////////////////////////////////////////////////////

use crate::prelude::*;

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

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Score(ScoreValue);

impl Score {
    pub const fn min_value() -> ScoreValue {
        0.0
    }

    pub const fn max_value() -> ScoreValue {
        1.0
    }

    pub const fn default_value() -> ScoreValue {
        Self::max_value()
    }

    pub fn clamp_value(value: impl Into<ScoreValue>) -> ScoreValue {
        //value.clamp(Self::min(), Self::max())
        let value = value.into();
        value.min(Self::max_value()).max(Self::min_value())
    }

    pub fn clamp_from(value: impl Into<ScoreValue>) -> Score {
        Self::clamp_value(value).into()
    }

    pub const fn min() -> Self {
        Self(Self::min_value())
    }

    pub const fn max() -> Self {
        Self(Self::max_value())
    }

    pub const fn default() -> Self {
        Self(Self::default_value())
    }

    pub const fn new(inner: ScoreValue) -> Self {
        Self(inner)
    }

    pub const fn value(self) -> ScoreValue {
        let Self(value) = self;
        value
    }

    // Convert to percentage value with a single decimal digit
    pub fn to_percentage(self) -> ScoreValue {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
    pub fn to_permille(self) -> u16 {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() as u16
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::default()
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
        from.value()
    }
}

impl From<ScoreValue> for Score {
    fn from(value: ScoreValue) -> Self {
        Self::new(value)
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
    pub fn clamp_str(value: &str) -> &str {
        value.trim()
    }

    pub fn clamp_value(value: impl Into<LabelValue>) -> LabelValue {
        let value = value.into();
        let clamped = Self::clamp_str(&value);
        if clamped == &value {
            value
        } else {
            clamped.into()
        }
    }

    pub fn clamp_from(value: impl Into<LabelValue>) -> Label {
        Self::clamp_value(value).into()
    }

    pub const fn new(value: LabelValue) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }

    pub fn into_value(self) -> LabelValue {
        let Self(value) = self;
        value
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
            .invalidate_if(self.value().is_empty(), LabelInvalidity::Empty)
            .invalidate_if(
                Self::clamp_str(self.as_ref()) != self.value().as_str(),
                LabelInvalidity::Format,
            )
            .into()
    }
}

impl From<LabelValue> for Label {
    fn from(value: LabelValue) -> Self {
        Self::new(value)
    }
}

impl From<Label> for LabelValue {
    fn from(from: Label) -> Self {
        from.into_value()
    }
}

impl AsRef<LabelValue> for Label {
    fn as_ref(&self) -> &LabelValue {
        let Self(value) = self;
        value
    }
}

impl AsRef<str> for Label {
    fn as_ref(&self) -> &str {
        let Self(value) = self;
        value
    }
}

impl FromStr for Label {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::clamp_str(s).to_owned().into())
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
/// See also: Spotify categories
///
/// Format: ASCII string, no uppercase characters, no whitespace
#[derive(Clone, Default, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Facet(FacetValue);

impl Facet {
    pub fn clamp_value(value: impl Into<FacetValue>) -> FacetValue {
        let mut value = value.into();
        value.retain(Self::is_valid_char);
        value
    }

    pub fn clamp_from(value: impl Into<FacetValue>) -> Label {
        Self::clamp_value(value).into()
    }

    pub const fn new(value: FacetValue) -> Self {
        Self(value)
    }

    pub fn into_value(self) -> FacetValue {
        let Self(value) = self;
        value
    }

    pub const fn value(&self) -> &FacetValue {
        let Self(value) = self;
        value
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
            .invalidate_if(self.value().is_empty(), FacetInvalidity::Empty)
            .invalidate_if(
                self.value().chars().any(Facet::is_invalid_char),
                FacetInvalidity::Format,
            )
            .into()
    }
}

impl From<FacetValue> for Facet {
    fn from(from: FacetValue) -> Self {
        Self::new(from)
    }
}

impl From<Facet> for FacetValue {
    fn from(from: Facet) -> Self {
        from.into_value()
    }
}

impl AsRef<FacetValue> for Facet {
    fn as_ref(&self) -> &FacetValue {
        let Self(value) = self;
        value
    }
}

impl AsRef<str> for Facet {
    fn as_ref(&self) -> &str {
        let Self(value) = self;
        value
    }
}

impl FromStr for Facet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::clamp_value(s).into())
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.value())
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
    pub const fn new(inner: Option<Facet>) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> Option<Facet> {
        let Self(inner) = self;
        inner
    }
}

impl From<Option<Facet>> for FacetKey {
    fn from(inner: Option<Facet>) -> Self {
        FacetKey::new(inner)
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
        let Self(inner) = self;
        inner
    }
}

impl AsRef<str> for FacetKey {
    fn as_ref(&self) -> &str {
        let Self(inner) = self;
        match inner {
            Some(facet) => facet.as_ref(),
            None => "",
        }
    }
}

impl FromStr for FacetKey {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = Facet::clamp_value(s);
        let inner = if value.is_empty() {
            None
        } else {
            Some(Facet::new(value))
        };
        Ok(inner.into())
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
    pub const fn new(inner: TagsMap) -> Self {
        Self(inner)
    }

    pub fn into_inner(self) -> TagsMap {
        let Self(inner) = self;
        inner
    }

    pub fn insert(&mut self, key: FacetKey, tag: PlainTag) {
        use std::collections::hash_map::*;
        let Self(inner) = self;
        match inner.entry(key) {
            Entry::Occupied(mut entry) => {
                entry.get_mut().push(tag);
            }
            Entry::Vacant(entry) => {
                entry.insert(vec![tag]);
            }
        }
    }
}

impl From<TagsMap> for Tags {
    fn from(from: TagsMap) -> Self {
        Self::new(from)
    }
}

impl From<Tags> for TagsMap {
    fn from(from: Tags) -> Self {
        from.into_inner()
    }
}

impl AsRef<TagsMap> for Tags {
    fn as_ref(&self) -> &TagsMap {
        let Self(inner) = self;
        inner
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
        let Self(inner) = self;
        Self::validate_iter(ValidationContext::new(), inner.iter())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
