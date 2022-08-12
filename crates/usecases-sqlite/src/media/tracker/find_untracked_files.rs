// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_core_api::media::tracker::{find_untracked_files::Outcome, FsTraversalParams};

use super::*;

mod uc {
    pub(super) use aoide_usecases::media::tracker::find_untracked_files::*;
}

pub fn visit_directories<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &FsTraversalParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let mut repo = RepoConnection::new(connection);
    uc::visit_directories(
        &mut repo,
        collection_uid,
        params,
        report_progress_fn,
        abort_flag,
    )
    .map_err(Into::into)
}
