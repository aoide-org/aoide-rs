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

use aoide_core_api::{media::SyncMode, track::replace::Summary};

use aoide_media::io::import::ImportTrackConfig;

use aoide_repo::track::ReplaceMode;
use aoide_usecases::track::ValidatedInput;

use super::*;

mod uc {
    pub use aoide_usecases::track::replace::{
        import_and_replace_by_local_file_path_from_directory,
        import_and_replace_by_local_file_paths,
        replace_collected_tracks_by_media_source_content_path, Outcome, Params,
    };
}

pub fn replace_by_media_source_content_path(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &uc::Params,
    tracks: impl IntoIterator<Item = ValidatedInput>,
) -> Result<Summary> {
    let repo = RepoConnection::new(connection);
    uc::replace_collected_tracks_by_media_source_content_path(&repo, collection_uid, params, tracks)
        .map_err(Into::into)
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_paths(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_paths: impl IntoIterator<Item = ContentPath>,
    expected_source_path_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let repo = RepoConnection::new(connection);
    uc::import_and_replace_by_local_file_paths(
        &repo,
        collection_uid,
        sync_mode,
        import_config,
        replace_mode,
        source_paths,
        expected_source_path_count,
        abort_flag,
    )
    .map_err(Into::into)
}

pub fn import_and_replace_by_local_file_path_from_directory(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let repo = RepoConnection::new(connection);
    uc::import_and_replace_by_local_file_path_from_directory(
        &repo,
        collection_uid,
        sync_mode,
        import_config,
        replace_mode,
        source_dir_path,
        abort_flag,
    )
    .map_err(Into::into)
}
