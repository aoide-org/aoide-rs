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
/// Scoring
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
        debug_assert!(self.is_valid());
        (self.0 * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
    pub fn to_permille(self) -> u16 {
        debug_assert!(self.is_valid());
        (self.0 * ScoreValue::from(1_000)).round() as u16
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
        debug_assert!(new.is_valid());
        new
    }
}

impl IsValid for Score {
    fn is_valid(&self) -> bool {
        *self >= Self::min() && *self <= Self::max()
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        debug_assert!(self.is_valid());
        write!(f, "{:.1}%", self.to_percentage())
    }
}

///////////////////////////////////////////////////////////////////////
/// Label
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Label(String);

pub trait Labeled {
    fn label(&self) -> &Label;
}

impl Label {
    pub const fn new(label: String) -> Self {
        Self(label)
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
        debug_assert!(new.is_valid());
        new
    }
}

impl IsValid for Label {
    fn is_valid(&self) -> bool {
        !self.0.is_empty() && self.0.trim().len() == self.0.len()
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
/// Facet
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
        debug_assert!(new.is_valid());
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

impl IsValid for Facet {
    fn is_valid(&self) -> bool {
        !self.0.is_empty()
            && !self.0.chars().any(char::is_whitespace)
            && !self.0.chars().any(char::is_uppercase)
    }
}

impl fmt::Display for Facet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

///////////////////////////////////////////////////////////////////////
/// Tag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tag(Label, Score);

impl Tag {
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

impl Labeled for Tag {
    fn label(&self) -> &Label {
        &self.0
    }
}

impl Scored for Tag {
    fn score(&self) -> Score {
        self.1
    }
}

impl IsValid for Tag {
    fn is_valid(&self) -> bool {
        self.label().is_valid() && self.score().is_valid()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tags;

impl Tags {
    pub fn is_valid(slice: &[Tag]) -> bool {
        // TODO: Check for duplicate labels
        slice.iter().all(IsValid::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// FacetedTag
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct FacetedTag(Facet, Label, Score);

impl FacetedTag {
    pub const fn new(facet: Facet, label: Label, score: Score) -> Self {
        Self(facet, label, score)
    }
}

impl Faceted for FacetedTag {
    fn facet(&self) -> &Facet {
        &self.0
    }
}

impl Labeled for FacetedTag {
    fn label(&self) -> &Label {
        &self.1
    }
}

impl Scored for FacetedTag {
    fn score(&self) -> Score {
        self.2
    }
}

impl IsValid for FacetedTag {
    fn is_valid(&self) -> bool {
        self.facet().is_valid() && self.label().is_valid() && self.score().is_valid()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FacetedTags;

impl FacetedTags {
    pub fn is_valid(slice: &[FacetedTag]) -> bool {
        // TODO: Check for duplicate labels per facet
        slice.iter().all(IsValid::is_valid)
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
