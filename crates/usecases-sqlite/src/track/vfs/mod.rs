// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use aoide_core::CollectionUid;
use aoide_core_api::track::search::Filter;
use aoide_repo_sqlite::DbConnection;
use aoide_usecases::track::vfs::export_files as uc;

use crate::{RepoConnection, Result};

pub fn export_files(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    filter: Option<&Filter>,
    batch_size: Option<u64>,
    target_root_path: &Path,
    match_files: uc::MatchFiles,
    purge_other_files: bool,
) -> Result<uc::ExportTrackFilesOutcome> {
    let mut repo = RepoConnection::new(connection);
    uc::export_files(
        &mut repo,
        collection_uid,
        filter,
        batch_size,
        target_root_path,
        match_files,
        purge_other_files,
    )
    .map_err(Into::into)
}
