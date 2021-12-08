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

use aoide_usecases_sqlite::SqlitePooledConnection;

use aoide_core::{
    entity::EntityUid,
    track::tag::{FACET_GENRE, FACET_MOOD},
    util::url::BaseUrl,
};

use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use super::*;

mod uc {
    pub use aoide_core_ext::media::tracker::import::*;
    pub use aoide_usecases_sqlite::media::tracker::import::*;
}

pub type RequestBody = aoide_core_ext_serde::media::tracker::import::Params;

pub type ResponseBody = aoide_core_ext_serde::media::tracker::import::Outcome;

#[tracing::instrument(
    name = "Importing media sources",
    skip(
        pooled_connection,
        progress_summary_fn,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
    progress_summary_fn: &mut impl FnMut(&uc::Summary),
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
    // FIXME: Replace hard-coded import flags
    let import_flags = ImportTrackFlags::ARTWORK_DIGEST
        | ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK
        | ImportTrackFlags::AOIDE_TAGS
        | ImportTrackFlags::SERATO_MARKERS;
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config.into(),
        flags: import_flags,
    };
    let params = aoide_core_ext::media::tracker::import::Params {
        root_url,
        import_mode: import_mode.map(Into::into),
    };
    uc::import(
        &pooled_connection,
        collection_uid,
        &params,
        &import_config,
        progress_summary_fn,
        abort_flag,
    )
    .map(Into::into)
    .map_err(Into::into)
}
