// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[allow(clippy::too_many_arguments)] // TODO
pub fn import_and_replace_many_by_local_file_path<InterceptImportedTrackFn>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    content_path_iter: impl IntoIterator<Item = ContentPath<'static>>,
    expected_content_path_count: impl Into<Option<usize>>,
    params: &uc::Params,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome>
where
    InterceptImportedTrackFn: FnMut(Track) -> Track,
{
    let mut repo = RepoConnection::new(connection);
    uc::import_and_replace_many_by_local_file_path(
        &mut repo,
        collection_uid,
        params,
        content_path_iter,
        expected_content_path_count.into(),
        intercept_imported_track_fn,
        abort_flag,
    )
    .map_err(Into::into)
}

pub fn import_and_replace_by_local_file_path_from_directory<InterceptImportedTrackFn>(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    params: &uc::Params,
    source_dir_path: &ContentPath<'_>,
    intercept_imported_track_fn: &mut InterceptImportedTrackFn,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome>
where
    InterceptImportedTrackFn: FnMut(Track) -> Track + Send + Sync,
{
    let mut repo = RepoConnection::new(connection);
    uc::import_and_replace_by_local_file_path_from_directory(
        &mut repo,
        collection_uid,
        source_dir_path,
        params,
        intercept_imported_track_fn,
        abort_flag,
    )
    .map_err(Into::into)
}
