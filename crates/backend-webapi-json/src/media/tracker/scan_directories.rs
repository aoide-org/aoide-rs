// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::tracker::scan_directories::ProgressEvent;
    pub(super) use aoide_usecases_sqlite::media::tracker::scan_directories::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::FsTraversalParams;

pub type ResponseBody = aoide_core_api_json::media::tracker::scan_directories::Outcome;

#[tracing::instrument(
    name = "Scanning media sources",
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
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    connection
        .transaction::<_, Error, _>(|| {
            uc::scan_directories(
                connection,
                collection_uid,
                &params,
                report_progress_fn,
                abort_flag,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
