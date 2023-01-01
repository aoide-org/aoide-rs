// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_core::track::tag::{FACET_GENRE, FACET_MOOD};

use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::tracker::import_files::ProgressEvent;
    pub(super) use aoide_usecases_sqlite::media::tracker::import_files::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::import_files::Params;

pub type ResponseBody = aoide_core_api_json::media::tracker::import_files::Outcome;

#[allow(clippy::panic_in_result_fn)] // tracing::instrument
#[tracing::instrument(
    name = "Importing media sources",
    skip(
        connection,
        report_progress_fn,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    let mut faceted_tag_mapping_config = FacetedTagMappingConfigInner::default();
    faceted_tag_mapping_config.insert(
        FACET_GENRE.to_owned(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    faceted_tag_mapping_config.insert(
        FACET_MOOD.to_owned(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    // FIXME: Replace hard-coded import flags
    let import_flags = ImportTrackFlags::all();
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config.into(),
        flags: import_flags,
    };
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::import_files(
                connection,
                collection_uid,
                &params,
                import_config,
                report_progress_fn,
                abort_flag,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
