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

use super::*;

use aoide_core::track::tag::{FACET_GENRE, FACET_MOOD};

use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use std::sync::atomic::AtomicBool;
use tokio::sync::watch;
use url::Url;

mod _core {
    pub use aoide_core::{
        entity::EntityUid,
        usecases::media::tracker::import::{DirectorySummary, Summary},
    };
}

mod uc {
    pub use crate::usecases::media::tracker::{import::*, *};
    pub use aoide_usecases::media::tracker::import::Outcome;
}

pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
    usecases::media::tracker::import::Summary,
};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_mode: Option<ImportMode>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
}

impl From<uc::Outcome> for Outcome {
    fn from(from: uc::Outcome) -> Self {
        let uc::Outcome {
            completion,
            summary,
        } = from;
        Self {
            completion: completion.into(),
            summary: summary.into(),
        }
    }
}

pub type RequestBody = Params;

pub type ResponseBody = Outcome;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    request_body: RequestBody,
    progress_summary_tx: Option<&watch::Sender<_core::Summary>>,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        import_mode,
    } = request_body;
    let import_mode = import_mode.unwrap_or(ImportMode::Modified);
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
    uc::import(
        &pooled_connection,
        collection_uid,
        import_mode.into(),
        &import_config,
        import_flags,
        root_url.as_ref(),
        &mut |summary| {
            if let Some(progress_summary_tx) = progress_summary_tx {
                if progress_summary_tx.send(summary.to_owned()).is_err() {
                    log::error!("Failed to send progress summary");
                }
            }
        },
        abort_flag,
    )
    .map(Into::into)
    .map_err(Into::into)
}
