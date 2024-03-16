// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::HashMap, fmt};

use aoide_core::prelude::*;
use serde::{de::Visitor, Deserializer, Serializer};

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::tag::{
        FacetId, FacetKey, FacetedTags, Label, PlainTag, Score, Tags,
    };
}

#[derive(Debug, PartialEq, Eq, Hash)]
#[repr(transparent)]
#[cfg_attr(test, derive(Clone))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct FacetKey {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    inner: _core::FacetKey<'static>,
}

impl From<_core::FacetKey<'_>> for FacetKey {
    fn from(inner: _core::FacetKey<'_>) -> Self {
        Self {
            inner: inner.into_owned(),
        }
    }
}

impl From<FacetKey> for _core::FacetKey<'_> {
    fn from(from: FacetKey) -> Self {
        from.inner
    }
}

impl AsRef<_core::FacetKey<'static>> for FacetKey {
    fn as_ref(&self) -> &_core::FacetKey<'static> {
        &self.inner
    }
}

impl Serialize for FacetKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.inner.as_str())
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
        let inner = if _core::FacetKey::default().as_str() == s {
            // Special case: Tags without a facet are referred to by an empty string,
            // i.e. by the string representation of the default facet identifier.
            _core::FacetKey::default()
        } else {
            let facet_id = _core::FacetId::new_unchecked(s.into());
            if !facet_id.is_valid() {
                return Err(serde::de::Error::invalid_value(
                    serde::de::Unexpected::Str(s),
                    &self,
                ));
            }
            Some(facet_id.into_owned()).into()
        };
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
#[repr(transparent)]
#[cfg_attr(test, derive(Clone, PartialEq, Eq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct Label {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    inner: _core::Label<'static>,
}

impl From<_core::Label<'_>> for Label {
    fn from(inner: _core::Label<'_>) -> Self {
        Self {
            inner: inner.into_owned(),
        }
    }
}

impl From<Label> for _core::Label<'static> {
    fn from(from: Label) -> Self {
        from.inner
    }
}

impl AsRef<_core::Label<'static>> for Label {
    fn as_ref(&self) -> &_core::Label<'static> {
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
        let label = _core::Label::new(s.into());
        if !label.is_valid() {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Str(s),
                &self,
            ));
        }
        Ok(label.into_owned().into())
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
#[repr(transparent)]
#[cfg_attr(test, derive(Clone, PartialEq))]
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
        serializer.serialize_f64(self.0.value())
    }
}

struct ScoreVisitor;

impl<'de> Visitor<'de> for ScoreVisitor {
    type Value = Score;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Score")
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let score = _core::Score::new_unchecked(value);
        if !score.is_valid() {
            return Err(serde::de::Error::invalid_value(
                serde::de::Unexpected::Float(value),
                &self,
            ));
        }
        Ok(score.into())
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

impl From<PlainTag> for _core::PlainTag<'static> {
    #[allow(clippy::cast_precision_loss)]
    fn from(from: PlainTag) -> Self {
        use PlainTag as From;
        match from {
            From::Label(label) => Self {
                label: Some(label.into()),
                ..Default::default()
            },
            From::Score(score) => Self {
                score: score.into(),
                ..Default::default()
            },
            From::IntScoreFallback(iscore) => Self {
                score: _core::Score::new_unchecked(iscore as f64),
                ..Default::default()
            },
            From::LabelIntScoreFallback(label, iscore) => Self {
                label: Some(label.into()),
                score: _core::Score::new_unchecked(iscore as f64),
            },
            From::LabelScore(label, score) => Self {
                label: Some(label.into()),
                score: score.into(),
            },
        }
    }
}

impl From<_core::PlainTag<'_>> for PlainTag {
    fn from(from: _core::PlainTag<'_>) -> Self {
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
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Tags(TagsMap);

impl Tags {
    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<_core::Tags<'static>> for Tags {
    fn from(from: _core::Tags<'static>) -> Self {
        let mut into = HashMap::with_capacity(from.total_count());
        let _core::Tags {
            plain: plain_tags,
            facets,
        } = from;
        if !plain_tags.is_empty() {
            into.insert(
                _core::FacetKey::new(None).into(),
                plain_tags.into_iter().map(Into::into).collect(),
            );
        }
        for faceted_tags in facets {
            let _core::FacetedTags { facet_id, tags } = faceted_tags;
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

impl From<Tags> for _core::Tags<'static> {
    fn from(from: Tags) -> Self {
        let Tags(from) = from;
        let mut plain_tags = vec![];
        let mut facets = Vec::with_capacity(from.len());
        for (key, tags) in from {
            let tags = tags.into_iter().map(Into::into).collect();
            let FacetKey { inner } = key;
            if let Some(facet_id) = inner.into() {
                facets.push(_core::FacetedTags { facet_id, tags });
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
