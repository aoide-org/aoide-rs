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
pub enum ScoreValidation {
    OutOfRange,
}

impl Validate for Score {
    type Validation = ScoreValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            !(*self >= Self::min() && *self <= Self::max()),
            ScoreValidation::OutOfRange,
        );
        context.into_result()
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
    pub const fn new(label: String) -> Self {
        Self(label)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LabelValidation {
    Empty,
    LeadingOrTrailingWhitespace,
}

impl Validate for Label {
    type Validation = LabelValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(self.0.is_empty(), LabelValidation::Empty);
        context.add_violation_if(
            self.0.trim().len() != self.0.len(),
            LabelValidation::LeadingOrTrailingWhitespace,
        );
        context.into_result()
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

impl<T> From<T> for Label
where
    T: Into<String>,
{
    fn from(from: T) -> Self {
        Self::new(from.into())
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
    pub const fn new(label: String) -> Self {
        Self(label)
    }

    fn is_invalid_char(c: char) -> bool {
        c.is_whitespace() || c.is_uppercase()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FacetValidation {
    Empty,
    InvalidChars,
}

impl Validate for Facet {
    type Validation = FacetValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(self.0.is_empty(), FacetValidation::Empty);
        context.add_violation_if(
            self.0.chars().any(Facet::is_invalid_char),
            FacetValidation::InvalidChars,
        );
        context.into_result()
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

impl<T> From<T> for Facet
where
    T: Into<String>,
{
    fn from(from: T) -> Self {
        Self::new(from.into())
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
pub enum TagValidation {
    Facet(FacetValidation),
    Label(LabelValidation),
    Score(ScoreValidation),
    BothFacetAndLabelMissing,
}

impl Validate for Tag {
    type Validation = TagValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if self.facet.is_none() && self.label.is_none() {
            context.add_violation(TagValidation::BothFacetAndLabelMissing);
        } else {
            if let Some(ref facet) = self.facet {
                context.map_and_merge_result(facet.validate(), TagValidation::Facet);
            }
            if let Some(ref label) = self.label {
                context.map_and_merge_result(label.validate(), TagValidation::Label);
            }
        }
        context.map_and_merge_result(self.score.validate(), TagValidation::Score);
        context.into_result()
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
pub enum TagsValidation {
    Tag(TagValidation),
    PlainDuplicateLabels,
    FacetedDuplicateLabels,
}

impl Tags {
    pub fn validate<'a, I>(tags: I) -> ValidationResult<TagsValidation>
    where
        I: IntoIterator<Item = &'a Tag> + Copy,
    {
        let mut context = ValidationContext::default();
        for tag in tags.into_iter() {
            context.map_and_merge_result(tag.validate(), TagsValidation::Tag);
        }
        let (plain, faceted): (Vec<_>, Vec<_>) =
            tags.into_iter().partition(|tag| tag.facet.is_none());
        let mut plain_labels: Vec<_> = plain.iter().map(|tag| &tag.label).collect();
        plain_labels.sort_unstable();
        plain_labels.dedup();
        context.add_violation_if(
            plain_labels.len() < plain.len(),
            TagsValidation::PlainDuplicateLabels,
        );
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
                    context.add_violation(TagsValidation::FacetedDuplicateLabels);
                    break;
                }
            }
            i = j;
        }
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
