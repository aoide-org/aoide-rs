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
    Facet as TagFacet, FacetValue, Label as TagLabel, LabelValue, PlainTag, Score as TagScore,
    ScoreValue, TagsMap,
};

use semval::IsValid as _;
use std::{
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
}

pub type FacetedTagMappingConfigInner = HashMap<FacetValue, TagMappingConfig>;

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

pub fn import_faceted_tags(
    tags_map: &mut TagsMap,
    next_score_value: &mut ScoreValue,
    facet: &TagFacet,
    tag_mapping_config: Option<&TagMappingConfig>,
    label_value: impl Into<LabelValue>,
) -> usize {
    let mut import_count = 0;
    let label_value = label_value.into();
    if let Some(tag_mapping_config) = tag_mapping_config {
        if !tag_mapping_config.label_separator.is_empty() {
            for (_, split_label_value) in
                label_value.match_indices(&tag_mapping_config.label_separator)
            {
                match try_import_plain_tag(split_label_value, *next_score_value) {
                    Ok(plain_tag) => {
                        tags_map.insert(facet.to_owned().into(), plain_tag);
                        import_count += 1;
                        *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                    }
                    Err(plain_tag) => {
                        log::warn!("Failed to import faceted '{}' tag: {:?}", facet, plain_tag,);
                    }
                }
            }
        }
    }
    if import_count == 0 {
        match try_import_plain_tag(label_value, *next_score_value) {
            Ok(plain_tag) => {
                tags_map.insert(facet.to_owned().into(), plain_tag);
                import_count += 1;
                if let Some(tag_mapping_config) = tag_mapping_config {
                    *next_score_value = tag_mapping_config.next_score_value(*next_score_value);
                }
            }
            Err(plain_tag) => {
                log::warn!("Failed to import faceted '{}' tag: {:?}", facet, plain_tag,);
            }
        }
    }
    import_count
}
