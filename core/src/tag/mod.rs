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

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
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

impl Validate for Score {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !(*self >= Self::min() && *self <= Self::max()) {
            errors.add("score", ValidationError::new("invalid value"));
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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

impl Validate for Label {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.0.is_empty() {
            errors.add("label", ValidationError::new("is empty"));
        }
        if self.0.trim().len() != self.0.len() {
            errors.add(
                "label",
                ValidationError::new("contains leading or trailing whitespace characters"),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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
}

impl Validate for Facet {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if self.0.is_empty() {
            errors.add("facet", ValidationError::new("is empty"));
        }
        if self.0.chars().any(char::is_whitespace) {
            errors.add(
                "facet",
                ValidationError::new("contains whitespace character(s)"),
            );
        }
        if self.0.chars().any(char::is_uppercase) {
            errors.add(
                "facet",
                ValidationError::new("contains uppercase character(s)"),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
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

#[derive(Clone, Debug, PartialEq, Validate)]
#[validate(schema(function = "validate_tag_facet_label"))]
pub struct Tag {
    pub facet: Option<Facet>,
    pub label: Option<Label>,
    pub score: Score,
}

impl Tag {
    pub const fn default_score() -> Score {
        Score::max()
    }
}

fn validate_tag_facet_label(tag: &Tag) -> Result<(), ValidationError> {
    if tag.facet.is_none() && tag.label.is_none() {
        return Err(ValidationError::new("both facet and label are missing"));
    }
    Ok(())
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

#[cfg(test)]
mod tests;
