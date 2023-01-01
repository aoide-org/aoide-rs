// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::HashMap, fmt};

use serde::{de::Visitor, Deserializer, Serializer};

use aoide_core::tag::FacetedTags;

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::tag::{FacetId, FacetKey, Label, PlainTag, Score, Tags};
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Clone))]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct FacetKey {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    inner: _core::FacetKey,
}

impl From<_core::FacetKey> for FacetKey {
    fn from(inner: _core::FacetKey) -> Self {
        Self { inner }
    }
}

impl From<FacetKey> for _core::FacetKey {
    fn from(from: FacetKey) -> Self {
        from.inner
    }
}

impl AsRef<_core::FacetKey> for FacetKey {
    fn as_ref(&self) -> &_core::FacetKey {
        &self.inner
    }
}

impl Serialize for FacetKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.inner.as_ref())
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
        let inner = _core::FacetId::clamp_from(s).into();
        Ok(FacetKey { inner })
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

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct Label {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    inner: _core::Label,
}

impl From<_core::Label> for Label {
    fn from(inner: _core::Label) -> Self {
        Self { inner }
    }
}

impl From<Label> for _core::Label {
    fn from(from: Label) -> Self {
        from.inner
    }
}

impl AsRef<_core::Label> for Label {
    fn as_ref(&self) -> &_core::Label {
        &self.inner
    }
}

impl Serialize for Label {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.inner.as_ref())
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

#[derive(Debug)]
#[cfg_attr(test, derive(Clone, PartialEq))]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(with = "f64"))]
pub struct Score(_core::Score);

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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Clone, PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
        for faceted_tags in facets {
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
        for (key, tags) in from {
            let tags = tags.into_iter().map(Into::into).collect();
            let FacetKey { inner } = key;
            if let Some(facet_id) = inner.into() {
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
