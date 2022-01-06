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

use std::{collections::HashMap, fmt};

use schemars::{gen::SchemaGenerator, schema::Schema};
use serde::{de::Visitor, Deserializer, Serializer};

use aoide_core::tag::FacetedTags;

use crate::prelude::*;

mod _core {
    pub use aoide_core::tag::{FacetId, FacetKey, Label, PlainTag, Score, Tags};
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct FacetKey(_core::FacetKey);

impl JsonSchema for FacetKey {
    fn schema_name() -> String {
        "FacetKey".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.subschema_for::<String>()
    }
}

impl From<_core::FacetKey> for FacetKey {
    fn from(from: _core::FacetKey) -> Self {
        Self(from)
    }
}

impl From<FacetKey> for _core::FacetKey {
    fn from(from: FacetKey) -> Self {
        from.0
    }
}

impl AsRef<_core::FacetKey> for FacetKey {
    fn as_ref(&self) -> &_core::FacetKey {
        &self.0
    }
}

impl Serialize for FacetKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.as_ref())
    }
}

struct FacetKeyVisitor;

impl<'de> Visitor<'de> for FacetKeyVisitor {
    type Value = FacetKey;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FacetKey")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(FacetKey(_core::FacetId::clamp_from(s).into()))
    }
}

impl<'de> Deserialize<'de> for FacetKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(FacetKeyVisitor)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Label(_core::Label);

impl JsonSchema for Label {
    fn schema_name() -> String {
        "Label".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.subschema_for::<String>()
    }
}

impl From<_core::Label> for Label {
    fn from(from: _core::Label) -> Self {
        Self(from)
    }
}

impl From<Label> for _core::Label {
    fn from(from: Label) -> Self {
        from.0
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

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(label) = _core::Label::clamp_from(s) {
            Ok(label.into())
        } else {
            Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &self,
            ))
        }
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

impl JsonSchema for Score {
    fn schema_name() -> String {
        "Score".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.subschema_for::<f64>()
    }
}

impl From<_core::Score> for Score {
    fn from(from: _core::Score) -> Self {
        Self(from)
    }
}

impl From<Score> for _core::Score {
    fn from(from: Score) -> Self {
        from.0
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

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(untagged, deny_unknown_fields)]
pub enum PlainTag {
    Label(Label),
    Score(Score),
    LabelScore(Label, Score),
    // Needed as a fallback to parse integer score values!
    IntScoreFallback(i64),
    LabelIntScoreFallback(Label, i64),
}

impl From<PlainTag> for _core::PlainTag {
    fn from(from: PlainTag) -> Self {
        use PlainTag::*;
        match from {
            Label(label) => _core::PlainTag {
                label: Some(label.into()),
                ..Default::default()
            },
            Score(score) => _core::PlainTag {
                score: score.into(),
                ..Default::default()
            },
            IntScoreFallback(iscore) => _core::PlainTag {
                score: _core::Score::new(iscore as f64),
                ..Default::default()
            },
            LabelIntScoreFallback(label, iscore) => _core::PlainTag {
                label: Some(label.into()),
                score: _core::Score::new(iscore as f64),
            },
            LabelScore(label, score) => _core::PlainTag {
                label: Some(label.into()),
                score: score.into(),
            },
        }
    }
}

impl From<_core::PlainTag> for PlainTag {
    fn from(from: _core::PlainTag) -> Self {
        let _core::PlainTag { label, score } = from;
        match (label, score) {
            (None, score) => Self::Score(score.into()),
            (Some(label), score) => {
                if score == _core::Score::default() {
                    Self::Label(label.into())
                } else {
                    Self::LabelScore(label.into(), score.into())
                }
            }
        }
    }
}

pub type TagsMap = HashMap<FacetKey, Vec<PlainTag>>;

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct Tags(TagsMap);

impl Tags {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<_core::Tags> for Tags {
    fn from(from: _core::Tags) -> Self {
        let mut into = HashMap::with_capacity(from.total_count());
        let _core::Tags {
            plain: plain_tags,
            facets,
        } = from;
        if !plain_tags.is_empty() {
            into.insert(
                _core::FacetKey::from(None).into(),
                plain_tags.into_iter().map(Into::into).collect(),
            );
        }
        for faceted_tags in facets.into_iter() {
            let FacetedTags { facet_id, tags } = faceted_tags;
            if !tags.is_empty() {
                into.insert(
                    _core::FacetKey::from(facet_id).into(),
                    tags.into_iter().map(Into::into).collect(),
                );
            }
        }
        Self(into)
    }
}

impl From<Tags> for _core::Tags {
    fn from(from: Tags) -> Self {
        let Tags(from) = from;
        let mut plain_tags = vec![];
        let mut facets = Vec::with_capacity(from.len());
        for (key, tags) in from.into_iter() {
            let tags = tags.into_iter().map(Into::into).collect();
            let FacetKey(key) = key;
            if let Some(facet_id) = key.into() {
                facets.push(FacetedTags { facet_id, tags })
            } else {
                debug_assert!(plain_tags.is_empty());
                plain_tags = tags;
            }
        }
        _core::Tags {
            plain: plain_tags,
            facets,
        }
    }
}

#[cfg(test)]
mod tests;
