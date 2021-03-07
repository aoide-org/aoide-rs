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

use std::sync::atomic::AtomicBool;
use url::Url;

mod uc {
    pub use aoide_usecases::{
        collection::resolve_local_file_collection_id,
        media::{
            tracker::{import::*, *},
            *,
        },
        Error,
    };
}

pub use uc::Summary;

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    import_mode: ImportMode,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    root_dir_url: Option<&Url>,
    progress_fn: &mut impl FnMut(&Summary),
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    Ok(
        db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
            let (collection_id, source_path_resolver) =
                uc::resolve_local_file_collection_id(&db, collection_uid)
                    .map_err(DieselTransactionError::new)?;
            Ok(uc::import(
                &db,
                collection_id,
                import_mode,
                import_config,
                import_flags,
                &source_path_resolver,
                root_dir_url,
                progress_fn,
                abort_flag,
            )
            .map_err(DieselTransactionError::new)?)
        })?,
    )
}
