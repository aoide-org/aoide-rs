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

mod _core {
    pub use aoide_core::tag::*;
}

use serde::{de::Visitor, Deserializer, Serializer};
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Facet(_core::Facet);

impl Facet {
    pub fn into_inner(self) -> _core::Facet {
        self.0
    }
}

impl From<_core::Facet> for Facet {
    fn from(from: _core::Facet) -> Self {
        Self(from)
    }
}

impl AsRef<_core::Facet> for Facet {
    fn as_ref(&self) -> &_core::Facet {
        &self.0
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

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Facet")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<_core::Facet>()
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label(_core::Label);

impl Label {
    pub fn into_inner(self) -> _core::Label {
        self.0
    }
}

impl From<_core::Label> for Label {
    fn from(from: _core::Label) -> Self {
        Self(from)
    }
}

impl AsRef<_core::Label> for Label {
    fn as_ref(&self) -> &_core::Label {
        &self.0
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

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Label")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        v.parse::<_core::Label>()
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

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Score(_core::Score);

impl Score {
    pub fn into_inner(self) -> _core::Score {
        self.0
    }
}

impl From<_core::Score> for Score {
    fn from(from: _core::Score) -> Self {
        Self(from)
    }
}

impl AsRef<_core::Score> for Score {
    fn as_ref(&self) -> &_core::Score {
        &self.0
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

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Score")
    }

    fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(_core::Score::from(v).into())
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
    // Needed as a fallback to parse integer score values!
    LabelIntFallback(Label, i64),
}

impl From<PlainTag> for _core::Tag {
    fn from(from: PlainTag) -> Self {
        use PlainTag::*;
        match from {
            Label(label) => _core::Tag {
                label: Some(label.into_inner()),
                ..Default::default()
            },
            LabelScore(label, score) => _core::Tag {
                label: Some(label.into_inner()),
                score: score.into_inner(),
                ..Default::default()
            },
            LabelIntFallback(label, iscore) => _core::Tag {
                label: Some(label.into_inner()),
                score: _core::Score::new(iscore as f64),
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
    // Needed as a fallback to parse integer score values!
    FacetIntFallback(Facet, i64),
    FacetLabelIntFallback(Facet, Label, i64),
}

impl From<FacetedTag> for _core::Tag {
    fn from(from: FacetedTag) -> Self {
        use FacetedTag::*;
        match from {
            Facet(facet) => _core::Tag {
                facet: Some(facet.into_inner()),
                ..Default::default()
            },
            FacetScore(facet, score) => _core::Tag {
                facet: Some(facet.into_inner()),
                score: score.into_inner(),
                ..Default::default()
            },
            FacetLabel(facet, label) => _core::Tag {
                facet: Some(facet.into_inner()),
                label: Some(label.into_inner()),
                ..Default::default()
            },
            FacetLabelScore(facet, label, score) => _core::Tag {
                facet: Some(facet.into_inner()),
                label: Some(label.into_inner()),
                score: score.into_inner(),
            },
            FacetIntFallback(facet, iscore) => _core::Tag {
                facet: Some(facet.into_inner()),
                score: _core::Score::new(iscore as f64),
                ..Default::default()
            },
            FacetLabelIntFallback(facet, label, iscore) => _core::Tag {
                facet: Some(facet.into_inner()),
                label: Some(label.into_inner()),
                score: _core::Score::new(iscore as f64),
            },
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Tags(Vec<PlainTag>, Vec<FacetedTag>);

impl Tags {
    pub fn decode(self) -> Vec<_core::Tag> {
        let mut decoded = Vec::with_capacity(self.0.len() + self.1.len());
        for plain_tag in self.0.into_iter() {
            decoded.push(plain_tag.into());
        }
        for faceted_tag in self.1.into_iter() {
            decoded.push(faceted_tag.into());
        }
        decoded
    }

    pub fn encode(tags: Vec<_core::Tag>) -> Self {
        // Reserve the full capacity for both plain and faceted tags
        // to avoid reallocations during the encoding. Half of the
        // space will remain unused, but the doesn't matter in a
        // data transfer object with a very limited lifetime.
        let mut plain_tags = Vec::with_capacity(tags.len());
        let mut faceted_tags = Vec::with_capacity(tags.len());
        for tag in tags.into_iter() {
            let _core::Tag {
                facet,
                label,
                score,
            } = tag;
            match (facet, label) {
                (None, None) => unreachable!(), // invalid tag
                (None, Some(label)) => {
                    if score == _core::Tag::default_score() {
                        plain_tags.push(PlainTag::Label(label.into()));
                    } else {
                        plain_tags.push(PlainTag::LabelScore(label.into(), score.into()));
                    }
                }
                (Some(facet), None) => {
                    if score == _core::Tag::default_score() {
                        faceted_tags.push(FacetedTag::Facet(facet.into()));
                    } else {
                        faceted_tags.push(FacetedTag::FacetScore(facet.into(), score.into()));
                    }
                }
                (Some(facet), Some(label)) => faceted_tags.push(FacetedTag::FacetLabelScore(
                    facet.into(),
                    label.into(),
                    score.into(),
                )),
            }
        }
        Self(plain_tags, faceted_tags)
    }
}

#[cfg(test)]
mod tests;
