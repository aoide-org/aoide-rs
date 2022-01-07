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

use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use aoide_core::tag::{FacetIdValue, LabelValue, Score as TagScore, ScoreValue};

#[derive(Debug, Clone, PartialEq)]
pub struct TagMappingConfig {
    pub label_separator: LabelValue,
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
        labels: impl IntoIterator<Item = &'label str>,
        separator: impl AsRef<str>,
    ) -> Option<Cow<'label, str>> {
        let separator = separator.as_ref();
        debug_assert!(!separator.is_empty());
        labels.into_iter().fold(None, |joined_labels, next_label| {
            if let Some(joined_labels) = joined_labels {
                if next_label.is_empty() {
                    return Some(joined_labels);
                }
                let mut joined_labels: String = joined_labels.to_owned().into();
                joined_labels.push_str(separator);
                joined_labels.push_str(next_label);
                Some(joined_labels.into())
            } else {
                if next_label.is_empty() {
                    return None;
                }
                Some(next_label.into())
            }
        })
    }

    pub fn join_labels<'label>(
        &self,
        labels: impl IntoIterator<Item = &'label str>,
    ) -> Option<Cow<'label, str>> {
        Self::join_labels_with_separator(labels, &self.label_separator)
    }
}

pub type FacetedTagMappingConfigInner = HashMap<FacetIdValue, TagMappingConfig>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacetedTagMappingConfig(FacetedTagMappingConfigInner);

impl FacetedTagMappingConfig {
    #[must_use]
    pub const fn new(inner: FacetedTagMappingConfigInner) -> Self {
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
