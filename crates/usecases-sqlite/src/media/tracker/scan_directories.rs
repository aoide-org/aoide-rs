// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_core::CollectionUid;
use aoide_core_api::media::tracker::{FsTraversalParams, scan_directories::Outcome};
use aoide_repo_sqlite::DbConnection;

use crate::{RepoConnection, Result};

mod uc {
    pub(super) use aoide_usecases::media::tracker::scan_directories::*;
}

pub fn scan_directories<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &FsTraversalParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let mut repo = RepoConnection::new(connection);
    uc::scan_directories(
        &mut repo,
        collection_uid,
        params,
        report_progress_fn,
        abort_flag,
    )
    .map_err(Into::into)
}
