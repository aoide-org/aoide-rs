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

use super::*;

use aoide_core::{media::SourcePath, usecases::media::ImportMode};
use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    resolver::SourcePathResolver as _,
};
use aoide_repo::{collection::EntityRepo as _, track::ReplaceMode};
use aoide_usecases::collection::resolve_collection_id_for_virtual_file_path;

use std::sync::atomic::AtomicBool;

mod uc {
    pub use aoide_usecases::{
        collection::resolve_collection_id_for_virtual_file_path, track::replace::*, Error,
    };
}

pub fn replace_by_media_source_path(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &uc::Params,
    tracks: impl Iterator<Item = Track>,
) -> Result<uc::Summary> {
    let uc::Params {
        mode: replace_mode,
        resolve_path_from_url,
        preserve_collected_at,
    } = params;
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (collection_id, virtual_file_path_resolver) = if *resolve_path_from_url {
            let (collection_id, virtual_file_path_resolver) =
                resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                    .map_err(DieselTransactionError::new)?;
            (collection_id, Some(virtual_file_path_resolver))
        } else {
            let collection_id = db.resolve_collection_id(collection_uid)?;
            (collection_id, None)
        };
        let mut summary = uc::Summary::default();
        for mut track in tracks {
            if let Some(virtual_file_path_resolver) = virtual_file_path_resolver.as_ref() {
                let url = track
                    .media_source
                    .path
                    .parse()
                    .map_err(|err| {
                        anyhow::anyhow!(
                            "Failed to parse URL from path '{}': {}",
                            track.media_source.path,
                            err
                        )
                    })
                    .map_err(uc::Error::from)
                    .map_err(DieselTransactionError::new)?;
                track.media_source.path = virtual_file_path_resolver
                    .resolve_path_from_url(&url)
                    .map_err(|err| {
                        anyhow::anyhow!(
                            "Failed to resolve local file path from URL '{}': {}",
                            url,
                            err
                        )
                    })
                    .map_err(uc::Error::from)
                    .map_err(DieselTransactionError::new)?;
            }
            uc::replace_collected_track_by_media_source_path(
                &mut summary,
                &db,
                collection_id,
                *replace_mode,
                *preserve_collected_at,
                track,
            )?;
        }
        Ok(summary)
    })
    .map_err(Into::into)
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_path_iter(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
    source_path_iter: impl Iterator<Item = SourcePath>,
    expected_source_path_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (collection_id, source_path_resolver) =
            uc::resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                .map_err(DieselTransactionError::new)?;
        uc::import_and_replace_by_local_file_path_iter(
            &db,
            collection_id,
            import_mode,
            import_config,
            import_flags,
            replace_mode,
            &source_path_resolver,
            source_path_iter,
            expected_source_path_count,
            abort_flag,
        )
        .map_err(Into::into)
    })
    .map_err(Into::into)
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_local_file_path_from_directory(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
    source_dir_path: &str,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (collection_id, source_path_resolver) =
            uc::resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                .map_err(DieselTransactionError::new)?;
        uc::import_and_replace_by_local_file_path_from_directory(
            &db,
            collection_id,
            import_mode,
            import_config,
            import_flags,
            replace_mode,
            &source_path_resolver,
            source_dir_path,
            abort_flag,
        )
        .map_err(DieselTransactionError::new)
    })
    .map_err(Into::into)
}
