// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::media::SourcePath;

use aoide_core_api::{media::SyncMode, track::replace::Summary};

use aoide_media::io::import::ImportTrackConfig;

use aoide_repo::track::ReplaceMode;
use aoide_usecases::track::ValidatedInput;

use super::*;

mod uc {
    pub use aoide_usecases::track::replace::{
        import_and_replace_by_local_file_path_from_directory,
        import_and_replace_by_local_file_path_iter, replace_collected_tracks_by_media_source_path,
        Outcome, Params,
    };
}

pub fn replace_by_media_source_path(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &uc::Params,
    tracks: impl Iterator<Item = ValidatedInput>,
) -> Result<Summary> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::replace_collected_tracks_by_media_source_path(&db, collection_uid, params, tracks)
            .map_err(transaction_error)
    })
    .map_err(Into::into)
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_path_iter(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_path_iter: impl Iterator<Item = SourcePath>,
    expected_source_path_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::import_and_replace_by_local_file_path_iter(
            &db,
            collection_uid,
            sync_mode,
            import_config,
            replace_mode,
            source_path_iter,
            expected_source_path_count,
            abort_flag,
        )
        .map_err(transaction_error)
    })
    .map_err(Into::into)
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_path_from_directory(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    sync_mode: SyncMode,
    import_config: &ImportTrackConfig,
    replace_mode: ReplaceMode,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::import_and_replace_by_local_file_path_from_directory(
            &db,
            collection_uid,
            sync_mode,
            import_config,
            replace_mode,
            source_dir_path,
            abort_flag,
        )
        .map_err(transaction_error)
    })
    .map_err(Into::into)
}
