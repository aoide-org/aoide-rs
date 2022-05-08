// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::sync::atomic::AtomicBool;

use aoide_core::media::content::ContentPath;

use super::*;

mod uc {
    pub(super) use aoide_usecases::track::{
        import_and_replace::{
            import_and_replace_by_local_file_path_from_directory,
            import_and_replace_many_by_local_file_path, Params,
        },
        replace::Outcome,
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
