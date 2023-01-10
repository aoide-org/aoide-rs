// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use aoide_core::tag::{FacetKey, Score as TagScore, ScoreValue};

#[derive(Debug, Clone, PartialEq)]
pub struct TagMappingConfig {
    pub label_separator: String,
    pub split_score_attenuation: ScoreValue,
}

impl TagMappingConfig {
    #[must_use]
    pub fn next_score_value(&self, score: ScoreValue) -> ScoreValue {
        debug_assert!(score > TagScore::min().value());
        debug_assert!(self.split_score_attenuation > TagScore::min().value());
        score * self.split_score_attenuation
    }

    pub fn join_labels_with_separator<'label>(
        labels: impl IntoIterator<Item = Cow<'label, str>>,
        separator: impl AsRef<str>,
    ) -> Option<Cow<'label, str>> {
        let separator = separator.as_ref();
        debug_assert!(!separator.is_empty());
        labels.into_iter().fold(None, |joined_labels, next_label| {
            if let Some(joined_labels) = joined_labels {
                if next_label.is_empty() {
                    return Some(joined_labels);
                }
                let mut joined_labels: String = joined_labels.into_owned();
                joined_labels.push_str(separator);
                joined_labels.push_str(&next_label);
                Some(joined_labels.into())
            } else {
                if next_label.is_empty() {
                    return None;
                }
                Some(next_label)
            }
        })
    }

    pub fn join_labels<'label>(
        &self,
        labels: impl IntoIterator<Item = Cow<'label, str>>,
    ) -> Option<Cow<'label, str>> {
        Self::join_labels_with_separator(labels, &self.label_separator)
    }
}

pub type FacetedTagMappingConfigInner = HashMap<FacetKey<'static>, TagMappingConfig>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacetedTagMappingConfig(FacetedTagMappingConfigInner);

impl FacetedTagMappingConfig {
    #[must_use]
    pub(crate) const fn new(inner: FacetedTagMappingConfigInner) -> Self {
        Self(inner)
    }
}

impl From<FacetedTagMappingConfigInner> for FacetedTagMappingConfig {
    fn from(inner: FacetedTagMappingConfigInner) -> Self {
        Self::new(inner)
    }
}

impl From<FacetedTagMappingConfig> for FacetedTagMappingConfigInner {
    fn from(outer: FacetedTagMappingConfig) -> Self {
        let FacetedTagMappingConfig(inner) = outer;
        inner
    }
}

impl Deref for FacetedTagMappingConfig {
    type Target = FacetedTagMappingConfigInner;

    fn deref(&self) -> &Self::Target {
        let Self(inner) = self;
        inner
    }
}

impl DerefMut for FacetedTagMappingConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let Self(inner) = self;
        inner
    }
}
