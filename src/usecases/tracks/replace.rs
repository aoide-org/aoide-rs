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

use aoide_media::io::import::{ImportTrackConfig, ImportTrackFlags};
use aoide_repo::{collection::EntityRepo as _, track::ReplaceMode};
use aoide_usecases::media::ImportMode;

use std::sync::atomic::AtomicBool;
use url::Url;

mod uc {
    pub use aoide_usecases::{tracks::replace::*, Error};
}

pub fn replace_by_media_source_uri(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    replace_mode: ReplaceMode,
    tracks: impl Iterator<Item = Track>,
) -> Result<uc::Summary> {
    let db = RepoConnection::new(connection);
    Ok(
        db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
            let collection_id = db.resolve_collection_id(collection_uid)?;
            let mut summary = uc::Summary::default();
            for track in tracks {
                uc::replace_collected_track_by_media_source_uri(
                    &mut summary,
                    &db,
                    collection_id,
                    replace_mode,
                    false,
                    track,
                )?;
            }
            Ok(summary)
        })?,
    )
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_media_source_uri(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
    file_uris: impl Iterator<Item = String>,
    file_uri_count: Option<usize>,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    Ok(
        db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
            let collection_id = db.resolve_collection_id(collection_uid)?;
            Ok(uc::import_and_replace_by_media_source_uri(
                &db,
                collection_id,
                import_mode,
                import_config,
                import_flags,
                replace_mode,
                file_uris,
                file_uri_count,
                abort_flag,
            )?)
        })?,
    )
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import_and_replace_by_media_source_uri_from_directory(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    dir_url: &Url,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    replace_mode: ReplaceMode,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    Ok(
        db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
            let collection_id = db.resolve_collection_id(collection_uid)?;
            uc::import_and_replace_by_media_source_uri_from_directory(
                &db,
                collection_id,
                dir_url,
                import_mode,
                import_config,
                import_flags,
                replace_mode,
                abort_flag,
            )
            .map_err(DieselTransactionError::new)
        })?,
    )
}
