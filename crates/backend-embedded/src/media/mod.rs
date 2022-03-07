// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
