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

use aoide_core::tag::{Facet as CoreFacet, Label as CoreLabel, Score as CoreScore, Tag as CoreTag};

use serde::{de::Visitor, Deserializer, Serializer};
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Facet(CoreFacet);

impl Facet {
    pub fn into_inner(self) -> CoreFacet {
        self.0
    }
}

impl From<CoreFacet> for Facet {
    fn from(from: CoreFacet) -> Self {
        Self(from)
    }
}

impl Serialize for Facet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

struct FacetVisitor;

impl<'de> Visitor<'de> for FacetVisitor {
    type Value = Facet;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Facet")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<CoreFacet>()
            .map(Into::into)
            .map_err(|()| serde::de::Error::custom("invalid tag facet"))
    }
}

impl<'de> Deserialize<'de> for Facet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FacetVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Label(CoreLabel);

impl Label {
    pub fn into_inner(self) -> CoreLabel {
        self.0
    }
}

impl From<CoreLabel> for Label {
    fn from(from: CoreLabel) -> Self {
        Self(from)
    }
}

impl Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

struct LabelVisitor;

impl<'de> Visitor<'de> for LabelVisitor {
    type Value = Label;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Label")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<CoreLabel>()
            .map(Into::into)
            .map_err(|()| serde::de::Error::custom("invalid tag label"))
    }
}

impl<'de> Deserialize<'de> for Label {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(LabelVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Score(CoreScore);

impl Score {
    pub fn into_inner(self) -> CoreScore {
        self.0
    }
}

impl From<CoreScore> for Score {
    fn from(from: CoreScore) -> Self {
        Self(from)
    }
}

impl Serialize for Score {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0.into())
    }
}

struct ScoreVisitor;

impl<'de> Visitor<'de> for ScoreVisitor {
    type Value = Score;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Score")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(CoreScore::from(v).into())
    }
}

impl<'de> Deserialize<'de> for Score {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_f64(ScoreVisitor)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PlainTag {
    Label(Label),
    LabelScore(Label, Score),
}

impl From<PlainTag> for CoreTag {
    fn from(from: PlainTag) -> Self {
        use PlainTag::*;
        match from {
            Label(label) => CoreTag {
                label: Some(label.into_inner()),
                ..Default::default()
            },
            LabelScore(label, score) => CoreTag {
                label: Some(label.into_inner()),
                score: score.into_inner(),
                ..Default::default()
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FacetedTag {
    Facet(Facet),
    FacetScore(Facet, Score),
    FacetLabel(Facet, Label),
    FacetLabelScore(Facet, Label, Score),
}

impl From<FacetedTag> for CoreTag {
    fn from(from: FacetedTag) -> Self {
        use FacetedTag::*;
        match from {
            Facet(facet) => CoreTag {
                facet: Some(facet.into_inner()),
                ..Default::default()
            },
            FacetScore(facet, score) => CoreTag {
                facet: Some(facet.into_inner()),
                score: score.into_inner(),
                ..Default::default()
            },
            FacetLabel(facet, label) => CoreTag {
                facet: Some(facet.into_inner()),
                label: Some(label.into_inner()),
                ..Default::default()
            },
            FacetLabelScore(facet, label, score) => CoreTag {
                facet: Some(facet.into_inner()),
                label: Some(label.into_inner()),
                score: score.into_inner(),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(untagged)] // not deserializable (ambiguous)
pub enum Tag {
    Plain(PlainTag),
    Faceted(FacetedTag),
}

impl From<PlainTag> for Tag {
    fn from(from: PlainTag) -> Self {
        Tag::Plain(from)
    }
}

impl From<FacetedTag> for Tag {
    fn from(from: FacetedTag) -> Self {
        Tag::Faceted(from)
    }
}

impl From<CoreTag> for Tag {
    fn from(from: CoreTag) -> Self {
        debug_assert!(from.validate().is_ok());
        let CoreTag {
            facet,
            label,
            score,
        } = from;
        match (facet, label) {
            (None, None) => unreachable!(), // invalid tag
            (None, Some(label)) => {
                if score == CoreTag::default_score() {
                    Tag::Plain(PlainTag::Label(label.into()))
                } else {
                    Tag::Plain(PlainTag::LabelScore(label.into(), score.into()))
                }
            }
            (Some(facet), None) => {
                if score == CoreTag::default_score() {
                    Tag::Faceted(FacetedTag::Facet(facet.into()))
                } else {
                    Tag::Faceted(FacetedTag::FacetScore(facet.into(), score.into()))
                }
            }
            (Some(facet), Some(label)) => Tag::Faceted(FacetedTag::FacetLabelScore(
                facet.into(),
                label.into(),
                score.into(),
            )),
        }
    }
}

impl From<Tag> for CoreTag {
    fn from(from: Tag) -> Self {
        match from {
            Tag::Plain(tag) => tag.into(),
            Tag::Faceted(tag) => tag.into(),
        }
    }
}

#[cfg(test)]
mod tests;
