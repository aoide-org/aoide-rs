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
    CowLabel, FacetId as TagFacetId, FacetIdValue, Label as TagLabel, LabelValue, PlainTag,
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

pub fn try_import_plain_tag<'a>(
    label: impl Into<Option<CowLabel<'a>>>,
    score_value: impl Into<ScoreValue>,
) -> StdResult<PlainTag, PlainTag> {
    let label = label.into().map(Into::into);
    let score = TagScore::clamp_from(score_value);
    let plain_tag = PlainTag { label, score };
    if plain_tag.is_valid() {
        Ok(plain_tag)
    } else {
        Err(plain_tag)
    }
}

pub fn import_plain_tags_from_joined_label_value<'a>(
    tag_mapping_config: Option<&TagMappingConfig>,
    next_score_value: &mut ScoreValue,
    plain_tags: &mut Vec<PlainTag>,
    joined_label_value: impl Into<Cow<'a, str>>,
) -> usize {
    if let Some(joined_label_value) = TagLabel::clamp_value(joined_label_value) {
        debug_assert!(!joined_label_value.is_empty());
        let mut import_count = 0;
        if let Some(tag_mapping_config) = tag_mapping_config {
            if !tag_mapping_config.label_separator.is_empty() {
                for label_value in joined_label_value.split(&tag_mapping_config.label_separator) {
                    let label = TagLabel::clamp_value(label_value);
                    match try_import_plain_tag(label, *next_score_value) {
                        Ok(plain_tag) => {
                            plain_tags.push(plain_tag);
                            import_count += 1;
                            *next_score_value =
                                tag_mapping_config.next_score_value(*next_score_value);
                        }
                        Err(plain_tag) => {
                            log::warn!("Failed to import plain tag: {:?}", plain_tag,);
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
                    log::warn!("Failed to import plain tag: {:?}", plain_tag,);
                }
            }
        }
        import_count
    } else {
        log::debug!("Skipping empty tag label");
        0
    }
}

pub fn import_faceted_tags_from_label_values(
    tags_map: &mut TagsMap,
    faceted_tag_mapping_config: &FacetedTagMappingConfig,
    facet_id: &TagFacetId,
    label_values: impl IntoIterator<Item = LabelValue>,
) -> usize {
    let tag_mapping_config = faceted_tag_mapping_config.get(facet_id.value());
    let mut plain_tags = Vec::with_capacity(8);
    let mut next_score_value = TagScore::default_value();
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
