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

///////////////////////////////////////////////////////////////////////

use super::*;

use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
// Scoring
///////////////////////////////////////////////////////////////////////

pub type ScoreValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Score(ScoreValue);

pub trait Scored {
    fn score(&self) -> Score;
}

impl Score {
    pub const fn min() -> Self {
        Self(0.0)
    }

    pub const fn max() -> Self {
        Self(1.0)
    }

    pub const fn new(score: ScoreValue) -> Self {
        Self(score)
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
        from.0
    }
}

impl From<ScoreValue> for Score {
    fn from(from: ScoreValue) -> Self {
        Self::new(from)
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.validate().is_ok());
        write!(f, "{:.1}%", self.to_percentage())
    }
}

///////////////////////////////////////////////////////////////////////
// Label
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(String);

pub trait Labeled {
    fn label(&self) -> Option<&Label>;
}

impl Label {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LabelInvalidity {
    Empty,
    LeadingOrTrailingWhitespace,
}

impl Validate for Label {
    type Invalidity = LabelInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0.is_empty(), LabelInvalidity::Empty)
            .invalidate_if(
                self.0.trim().len() != self.0.len(),
                LabelInvalidity::LeadingOrTrailingWhitespace,
            )
            .into()
    }
}

impl From<Label> for String {
    fn from(from: Label) -> Self {
        from.0
    }
}

impl AsRef<String> for Label {
    fn as_ref(&self) -> &String {
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
        let label = s.trim().to_string();
        Ok(Self(label))
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

///////////////////////////////////////////////////////////////////////
// Facet
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Facet(String);

pub trait Faceted {
    fn facet(&self) -> Option<&Facet>;
}

impl Facet {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    fn is_invalid_char(c: char) -> bool {
        c.is_whitespace() || c.is_uppercase()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FacetInvalidity {
    Empty,
    InvalidChars,
}

impl Validate for Facet {
    type Invalidity = FacetInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.0.is_empty(), FacetInvalidity::Empty)
            .invalidate_if(
                self.0.chars().any(Facet::is_invalid_char),
                FacetInvalidity::InvalidChars,
            )
            .into()
    }
}

impl From<Facet> for String {
    fn from(from: Facet) -> Self {
        from.0
    }
}

impl AsRef<String> for Facet {
    fn as_ref(&self) -> &String {
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
        let s = s.trim();
        let mut facet = String::with_capacity(s.len());
        for c in s.chars() {
            let lc = if c.is_whitespace() { '_' } else { c }.to_lowercase();
            for c in lc {
                facet.push(c)
            }
        }
        Ok(Self(facet))
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0)
    }
}

///////////////////////////////////////////////////////////////////////
// Tag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq)]
pub struct Tag {
    pub facet: Option<Facet>,
    pub label: Option<Label>,
    pub score: Score,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TagInvalidity {
    Facet(FacetInvalidity),
    Label(LabelInvalidity),
    Score(ScoreInvalidity),
    BothFacetAndLabelMissing,
}

impl Validate for Tag {
    type Invalidity = TagInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_and_map(&self.facet, TagInvalidity::Facet)
            .validate_and_map(&self.label, TagInvalidity::Label)
            .invalidate_if(
                self.facet.is_none() && self.label.is_none(),
                TagInvalidity::BothFacetAndLabelMissing,
            )
            .validate_and_map(&self.score, TagInvalidity::Score)
            .into()
    }
}

impl Tag {
    pub const fn default_score() -> Score {
        Score::max()
    }
}

impl Default for Tag {
    fn default() -> Self {
        Self {
            facet: None,
            label: None,
            score: Self::default_score(),
        }
    }
}

impl Faceted for Tag {
    fn facet(&self) -> Option<&Facet> {
        self.facet.as_ref()
    }
}

impl Labeled for Tag {
    fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }
}

impl Scored for Tag {
    fn score(&self) -> Score {
        self.score
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Tags;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TagsInvalidity {
    Tag(TagInvalidity),
    PlainDuplicateLabels,
    FacetedDuplicateLabels,
}

impl Tags {
    pub fn validate<'a, I>(tags: I) -> ValidationResult<TagsInvalidity>
    where
        I: Iterator<Item = &'a Tag> + Clone,
    {
        let mut context = tags.clone().fold(ValidationContext::new(), |context, tag| {
            context.validate_and_map(tag, TagsInvalidity::Tag)
        });
        let (plain, faceted): (Vec<_>, Vec<_>) = tags.partition(|tag| tag.facet.is_none());
        let mut grouped_by_facet = faceted.clone();
        grouped_by_facet.sort_unstable_by_key(|t| &t.facet);
        let mut i = 0;
        while i < grouped_by_facet.len() {
            let mut j = i + 1;
            while j < grouped_by_facet.len() {
                if grouped_by_facet[i].facet != grouped_by_facet[j].facet {
                    break;
                }
                j += 1;
            }
            if j <= grouped_by_facet.len() {
                debug_assert!(i < j);
                let mut faceted_labels: Vec<_> = grouped_by_facet[i..j]
                    .iter()
                    .map(|tag| &tag.label)
                    .collect();
                faceted_labels.sort_unstable();
                faceted_labels.dedup();
                if faceted_labels.len() < j - i {
                    context = context.invalidate(TagsInvalidity::FacetedDuplicateLabels);
                    break;
                }
            }
            i = j;
        }
        let mut plain_labels: Vec<_> = plain.iter().map(|tag| &tag.label).collect();
        plain_labels.sort_unstable();
        plain_labels.dedup();
        context
            .invalidate_if(
                plain_labels.len() < plain.len(),
                TagsInvalidity::PlainDuplicateLabels,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
