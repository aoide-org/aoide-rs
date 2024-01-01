// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_backend_embedded::media::predefined_faceted_tag_mapping_config;
use aoide_media_file::io::import::ImportTrackConfig;

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
    // FIXME: Replace hard-coded tag mapping config
    let faceted_tag_mapping_config = predefined_faceted_tag_mapping_config();
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config,
        ..Default::default()
    };
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::import_files(
                connection,
                collection_uid,
                &params,
                import_config,
                &mut std::convert::identity,
                report_progress_fn,
                abort_flag,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
