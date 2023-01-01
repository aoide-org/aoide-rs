// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_usecases::media::tracker::relink as uc;

use super::*;

pub fn relink_tracks_with_untracked_media_sources<ReportProgressFn: FnMut(&uc::Progress)>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    find_candidate_params: uc::FindCandidateParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Vec<uc::RelocatedMediaSource>> {
    let mut repo = RepoConnection::new(connection);
    uc::relink_tracks_with_untracked_media_sources(
        &mut repo,
        collection_uid,
        find_candidate_params,
        report_progress_fn,
        abort_flag,
    )
    .map_err(Into::into)
}
