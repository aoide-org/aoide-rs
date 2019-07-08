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

pub mod actor;
pub mod title;

use std::{fmt, str::FromStr};

///////////////////////////////////////////////////////////////////////
// Scoring
///////////////////////////////////////////////////////////////////////

pub type ScoreValue = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
        let new = Self::new(from);
        debug_assert!(new.validate().is_ok());
        new
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

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

impl From<Label> for String {
    fn from(from: Label) -> Self {
        from.0
    }
}

impl From<String> for Label {
    fn from(from: String) -> Self {
        let new = Self::new(from);
        debug_assert!(new.validate().is_ok());
        new
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
        write!(f, "{}", self.0)
    }
}

///////////////////////////////////////////////////////////////////////
// Facet
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Facet(String);

pub trait Faceted {
    fn facet(&self) -> &Facet;
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

impl From<Facet> for String {
    fn from(from: Facet) -> Self {
        from.0
    }
}

impl From<String> for Facet {
    fn from(from: String) -> Self {
        let new = Self::new(from);
        debug_assert!(new.validate().is_ok());
        new
    }
}

impl FromStr for Facet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        let mut facet = String::with_capacity(s.len());
        for c in s.chars() {
            let lc = if c.is_whitespace() {
                '_'.to_lowercase()
            } else {
                c.to_lowercase()
            };
            for c in lc {
                facet.push(c)
            }
        }
        Ok(Self(facet))
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

///////////////////////////////////////////////////////////////////////
// PlainTag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlainTag(Label, Score);

impl PlainTag {
    pub const fn default_score() -> Score {
        Score::max()
    }

    pub const fn new(label: Label, score: Score) -> Self {
        Self(label, score)
    }

    pub const fn new_label(label: Label) -> Self {
        Self(label, Self::default_score())
    }
}

impl Validate for PlainTag {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let res = ValidationErrors::merge(Ok(()), "plain tag", self.0.validate());
        ValidationErrors::merge(res, "plain tag", self.1.validate())
    }
}

impl Labeled for PlainTag {
    fn label(&self) -> Option<&Label> {
        Some(&self.0)
    }
}

impl Scored for PlainTag {
    fn score(&self) -> Score {
        self.1
    }
}

///////////////////////////////////////////////////////////////////////
// FacetedTag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacetedTag(Facet, Option<Label>, Score);

impl FacetedTag {
    pub const fn new(facet: Facet, label: Option<Label>, score: Score) -> Self {
        Self(facet, label, score)
    }
}

impl Faceted for FacetedTag {
    fn facet(&self) -> &Facet {
        &self.0
    }
}

impl Labeled for FacetedTag {
    fn label(&self) -> Option<&Label> {
        self.1.as_ref()
    }
}

impl Scored for FacetedTag {
    fn score(&self) -> Score {
        self.2
    }
}

impl Validate for FacetedTag {
    // TODO: Check for duplicate labels per facet
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut res = ValidationErrors::merge(Ok(()), "faceted tag", self.0.validate());
        if let Some(ref label) = self.1 {
            res = ValidationErrors::merge(res, "faceted tag", label.validate());
        }
        ValidationErrors::merge(res, "faceted tag", self.2.validate())
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, Validate)]
pub struct Tags {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[validate]
    pub plain: Vec<PlainTag>, // no duplicate labels allowed

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[validate]
    pub faceted: Vec<FacetedTag>, // no duplicate labels per facet allowed
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
