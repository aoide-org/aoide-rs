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

///////////////////////////////////////////////////////////////////////

use aoide_core::tag::{
    FacetId as TagFacetId, FacetIdValue, Label as TagLabel, LabelValue, PlainTag,
    Score as TagScore, ScoreValue, TagsMap,
};

use semval::IsValid as _;
use std::{
    borrow::Cow,
    collections::HashMap,
    ops::{Deref, DerefMut},
    result::Result as StdResult,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TagMappingConfig {
    pub label_separator: LabelValue,
    pub split_score_attenuation: ScoreValue,
}

impl TagMappingConfig {
    pub fn next_score_value(&self, score: ScoreValue) -> ScoreValue {
        debug_assert!(self.split_score_attenuation > TagScore::min().into());
        score * self.split_score_attenuation
    }

    pub fn join_labels_str_slice<'label>(
        &self,
        labels: &[&'label str],
    ) -> Option<Cow<'label, str>> {
        debug_assert!(!self.label_separator.is_empty());
        match labels {
            &[] => None,
            labels => Some(labels.join(&self.label_separator).into()),
        }
    }

    pub fn join_labels_str_iter<'label>(
        &self,
        labels: impl Iterator<Item = &'label str>,
    ) -> Option<Cow<'label, str>> {
        debug_assert!(!self.label_separator.is_empty());
        labels.fold(None, |joined_labels, next_label| {
            if let Some(joined_labels) = joined_labels {
                if next_label.is_empty() {
                    return Some(joined_labels);
                }
                let mut joined_labels: String = joined_labels.to_owned().into();
                joined_labels.push_str(&self.label_separator);
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
}

pub type FacetedTagMappingConfigInner = HashMap<FacetIdValue, TagMappingConfig>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FacetedTagMappingConfig(FacetedTagMappingConfigInner);

impl FacetedTagMappingConfig {
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

pub fn try_import_plain_tag(
    label_value: impl Into<LabelValue>,
    score_value: impl Into<ScoreValue>,
) -> StdResult<PlainTag, PlainTag> {
    let label = TagLabel::clamp_from(label_value);
    let score = TagScore::clamp_from(score_value);
    let plain_tag = PlainTag {
        label: Some(label),
        score,
    };
    if plain_tag.is_valid() {
        Ok(plain_tag)
    } else {
        Err(plain_tag)
    }
}

pub fn import_plain_tags_from_joined_label_value(
    tag_mapping_config: Option<&TagMappingConfig>,
    next_score_value: &mut ScoreValue,
    plain_tags: &mut Vec<PlainTag>,
    joined_label_value: impl Into<LabelValue>,
) -> usize {
    let joined_label_value = TagLabel::clamp_value(joined_label_value);
    if joined_label_value.is_empty() {
        tracing::debug!("Skipping empty tag label");
        return 0;
    }
    let mut import_count = 0;
    if let Some(tag_mapping_config) = tag_mapping_config {
        if !tag_mapping_config.label_separator.is_empty() {
            for label_value in joined_label_value
                .split(&tag_mapping_config.label_separator)
                .filter_map(|s| {
                    let s = TagLabel::clamp_str(s);
                    if s.is_empty() {
                        None
                    } else {
                        Some(s)
                    }
                })
            {
                match try_import_plain_tag(label_value, *next_score_value) {
                    Ok(plain_tag) => {
                        plain_tags.push(plain_tag);
                        import_count += 1;
                        *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                    }
                    Err(plain_tag) => {
                        tracing::warn!("Failed to import plain tag: {:?}", plain_tag,);
                    }
                }
            }
        }
    }
    if import_count == 0 {
        // Try to import the whole string as a single tag label
        match try_import_plain_tag(joined_label_value, *next_score_value) {
            Ok(plain_tag) => {
                plain_tags.push(plain_tag);
                import_count += 1;
                if let Some(tag_mapping_config) = tag_mapping_config {
                    *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                }
            }
            Err(plain_tag) => {
                tracing::warn!("Failed to import plain tag: {:?}", plain_tag,);
            }
        }
    }
    import_count
}

pub fn import_faceted_tags_from_label_value_iter(
    tags_map: &mut TagsMap,
    faceted_tag_mapping_config: &FacetedTagMappingConfig,
    facet_id: &TagFacetId,
    label_values: impl Iterator<Item = LabelValue>,
) -> usize {
    let tag_mapping_config = faceted_tag_mapping_config.get(facet_id.value());
    let mut plain_tags = Vec::with_capacity(8);
    let mut next_score_value = ScoreValue::default();
    for label_value in label_values {
        import_plain_tags_from_joined_label_value(
            tag_mapping_config,
            &mut next_score_value,
            &mut plain_tags,
            label_value,
        );
    }
    if plain_tags.is_empty() {
        return 0;
    }
    let count = plain_tags.len();
    tags_map.update_faceted_plain_tags_by_label_ordering(facet_id, plain_tags);
    count
}