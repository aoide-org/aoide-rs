// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_core::track::Track;
use aoide_core_api::media::tracker::import_files::Params;
use aoide_media_file::io::import::ImportTrackConfig;

use super::*;

mod uc {
    pub(super) use aoide_core_api::media::tracker::import_files::*;
    pub(super) use aoide_usecases::media::tracker::import_files::*;
}

pub fn import_files<InterceptImportedTrackFn, ReportProgressFn>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &Params,
    import_config: ImportTrackConfig,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome>
where
    InterceptImportedTrackFn: FnMut(Track) -> Track + Send + Sync,
    ReportProgressFn: FnMut(uc::ProgressEvent),
{
    let mut repo = RepoConnection::new(connection);
    let outcome = uc::import_files(
        &mut repo,
        collection_uid,
        params,
        import_config,
        intercept_imported_track_fn,
        report_progress_fn,
        abort_flag,
    )?;
    Ok(outcome)
}
