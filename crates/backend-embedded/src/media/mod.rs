// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::tag::{FACET_GENRE, FACET_MOOD};
use aoide_media::util::tag::{
    FacetedTagMappingConfig, FacetedTagMappingConfigInner, TagMappingConfig,
};

pub mod source;
pub mod tracker;

const DEFAULT_LABEL_SEPARATOR: &str = ";";

const DEFAULT_SCORE_ATTENUATION: f64 = 0.75;

// FIXME: Replace hard-coded tag mapping config
#[must_use]
pub fn predefined_faceted_tag_mapping_config() -> FacetedTagMappingConfig {
    let mut config = FacetedTagMappingConfigInner::default();
    config.insert(
        FACET_GENRE.to_owned(),
        TagMappingConfig {
            label_separator: DEFAULT_LABEL_SEPARATOR.to_owned(),
            split_score_attenuation: DEFAULT_SCORE_ATTENUATION,
        },
    );
    config.insert(
        FACET_MOOD.to_owned(),
        TagMappingConfig {
            label_separator: DEFAULT_LABEL_SEPARATOR.to_owned(),
            split_score_attenuation: DEFAULT_SCORE_ATTENUATION,
        },
    );
    config.into()
}
