// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_core::media::content::ContentPath;

use super::*;

mod uc {
    pub(super) use aoide_usecases::track::import_and_replace::{
        import_and_replace_by_local_file_path_from_directory,
        import_and_replace_many_by_local_file_path, Outcome, Params,
    };
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_many_by_local_file_path(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: &uc::Params,
    content_path_iter: impl IntoIterator<Item = ContentPath>,
    expected_content_path_count: impl Into<Option<usize>>,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let repo = RepoConnection::new(connection);
    uc::import_and_replace_many_by_local_file_path(
        &repo,
        collection_uid,
        params,
        content_path_iter,
        expected_content_path_count.into(),
        abort_flag,
    )
    .map_err(Into::into)
}

pub fn import_and_replace_by_local_file_path_from_directory(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: &uc::Params,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let repo = RepoConnection::new(connection);
    uc::import_and_replace_by_local_file_path_from_directory(
        &repo,
        collection_uid,
        params,
        source_dir_path,
        abort_flag,
    )
    .map_err(Into::into)
}
