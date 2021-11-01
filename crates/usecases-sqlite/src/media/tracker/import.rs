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

use aoide_core_ext::media::tracker::import::Params;

use aoide_media::io::import::{ImportTrackConfig, ImportTrackFlags};

use super::*;

mod uc {
    pub use aoide_core_ext::media::tracker::import::*;
    pub use aoide_usecases::{
        collection::resolve_collection_id_for_virtual_file_path,
        media::{
            tracker::{import::*, *},
            *,
        },
        Error,
    };
}

// TODO: Reduce number of arguments
#[allow(clippy::too_many_arguments)]
pub fn import(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &Params,
    import_config: &ImportTrackConfig,
    import_flags: ImportTrackFlags,
    progress_summary_fn: &mut impl FnMut(&uc::Summary),
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    let outcome = db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (collection_id, source_path_resolver) =
            uc::resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                .map_err(DieselTransactionError::new)?;
        uc::import(
            &db,
            &source_path_resolver,
            collection_id,
            params,
            import_config,
            import_flags,
            progress_summary_fn,
            abort_flag,
        )
        .map_err(DieselTransactionError::new)
    })?;
    tracing::info!("Analyzing and optimizing database after import finished");
    db.analyze_and_optimize_stats()?;
    Ok(outcome)
}
