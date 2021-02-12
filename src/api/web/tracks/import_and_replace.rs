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

use super::{
    replace_collected::{QueryParams, ReplaceMode},
    *,
};

mod uc {
    pub use crate::usecases::tracks::replace::*;
}

use aoide_core::track::tag::{FACET_GENRE, FACET_MOOD};
pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
};
use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

///////////////////////////////////////////////////////////////////////

pub type RequestBody = Vec<String>;

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResponseBody {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<Entity>,
    pub not_imported: Vec<String>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

impl From<uc::Outcome> for ResponseBody {
    fn from(from: uc::Outcome) -> Self {
        let uc::Outcome {
            created,
            updated,
            unchanged,
            not_imported,
            not_created,
            not_updated,
        } = from;
        Self {
            created: created.into_iter().map(Into::into).collect(),
            updated: updated.into_iter().map(Into::into).collect(),
            unchanged: unchanged.into_iter().map(Into::into).collect(),
            not_imported: not_imported.into_iter().map(Into::into).collect(),
            not_created: not_created.into_iter().map(Into::into).collect(),
            not_updated: not_updated.into_iter().map(Into::into).collect(),
        }
    }
}

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let QueryParams { mode } = query_params;
    let replace_mode = mode.unwrap_or(ReplaceMode::UpdateOrCreate);
    // FIXME: Replace hard-coded tag mapping config
    let mut faceted_tag_mapping_config = FacetedTagMappingConfigInner::default();
    faceted_tag_mapping_config.insert(
        FACET_GENRE.to_owned().into(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    faceted_tag_mapping_config.insert(
        FACET_MOOD.to_owned().into(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config.into(),
    };
    // FIXME: Replace hard-coded import flags
    let import_flags = ImportTrackFlags::ARTWORK_DIGEST
        | ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK
        | ImportTrackFlags::MIXXX_CUSTOM_TAGS
        | ImportTrackFlags::SERATO_TAGS;
    Ok(uc::import_and_replace_by_media_source_uri(
        &pooled_connection,
        collection_uid,
        &import_config,
        import_flags,
        replace_mode.into(),
        request_body.into_iter().map(Into::into),
    )
    .map(Into::into)?)
}
