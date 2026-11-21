// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

use aoide_core::CollectionUid;
use aoide_core_api::track::search::Filter;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::track::vfs::export_files::{ExportTrackFilesOutcome, MatchFiles};

use crate::Result;

pub async fn export_files(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    filter: Option<Filter>,
    batch_size: Option<u64>,
    target_root_path: PathBuf,
    match_files: MatchFiles,
    purge_other_files: bool,
) -> Result<ExportTrackFilesOutcome> {
    db_gatekeeper
        .spawn_blocking_read_transaction(move |connection| {
            aoide_usecases_sqlite::track::vfs::export_files(
                connection,
                &collection_uid,
                filter.as_ref(),
                batch_size,
                &target_root_path,
                match_files,
                purge_other_files,
            )
        })
        .await
}
