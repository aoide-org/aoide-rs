// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::tracker::find_untracked_files::ProgressEvent;
    pub(super) use aoide_usecases_sqlite::media::tracker::find_untracked_files::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::FsTraversalParams;

pub type ResponseBody = aoide_core_api_json::media::tracker::find_untracked_files::Outcome;

#[tracing::instrument(
    name = "Finding untracked media sources",
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
    connection: &mut SqliteConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let params = request_body
        .try_into()
        .map_err(Into::into)
        .map_err(Error::BadRequest)?;
    //FIXME: Add transactions after upgrading to diesel v2.0
    //connection.transaction::<_, Error, _>(|connection| {
    uc::visit_directories(
                connection,
                collection_uid,
                &params,
                report_progress_fn,
                abort_flag,
            )
            .map_err(Into::into)
        //})
        .map(Into::into)
}
