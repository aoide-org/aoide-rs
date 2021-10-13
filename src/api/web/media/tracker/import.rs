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

use std::sync::atomic::AtomicBool;

use tokio::sync::watch;

use aoide_core::{
    track::tag::{FACET_GENRE, FACET_MOOD},
    util::url::BaseUrl,
};

use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use aoide_core_ext_serde::media::{
    tracker::import::{Outcome, Params},
    ImportMode,
};

use super::*;

mod _inner {
    pub use aoide_core::entity::EntityUid;

    pub use aoide_core_ext::media::tracker::import::{DirectorySummary, Summary};
}

mod uc {
    pub use crate::usecases::media::tracker::import::*;
    pub use aoide_core_ext::media::tracker::import::*;
}

pub type RequestBody = Params;

pub type ResponseBody = Outcome;

#[tracing::instrument(
    name = "Importing media sources",
    skip(
        pooled_connection,
        progress_summary_tx,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_inner::EntityUid,
    request_body: RequestBody,
    progress_summary_tx: Option<&watch::Sender<_inner::Summary>>,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        import_mode,
    } = request_body;
    let root_url = root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
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
        root_url,
        &mut |summary| {
            if let Some(progress_summary_tx) = progress_summary_tx {
                if progress_summary_tx.send(summary.to_owned()).is_err() {
                    tracing::error!("Failed to send progress summary");
                }
            }
        },
        abort_flag,
    )
    .map(Into::into)
    .map_err(Into::into)
}
