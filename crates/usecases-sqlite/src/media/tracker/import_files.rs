// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_core_api::media::tracker::import_files::Params;
use aoide_media::io::import::ImportTrackConfig;

use super::*;

mod uc {
    pub(super) use aoide_core_api::media::tracker::import_files::*;
    pub(super) use aoide_usecases::media::tracker::import_files::*;
}

pub fn import_files<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &Params,
    import_config: ImportTrackConfig,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let mut repo = RepoConnection::new(connection);
    let outcome = uc::import_files(
        &mut repo,
        collection_uid,
        params,
        import_config,
        report_progress_fn,
        abort_flag,
    )?;
    Ok(outcome)
}
