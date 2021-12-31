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

use aoide_core::entity::EntityUid;
use aoide_core_api::media::tracker::import_files::Params;
use aoide_media::io::import::ImportTrackConfig;
use aoide_storage_sqlite::analyze_and_optimize_database_stats;

use super::*;

mod uc {
    pub use aoide_core_api::media::tracker::import_files::*;
    pub use aoide_usecases::media::{
        tracker::{import_files::*, *},
        *,
    };
}

pub fn import_files<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &Params,
    import_config: &ImportTrackConfig,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let db = RepoConnection::new(connection);
    let outcome = db.transaction::<_, TransactionError, _>(|| {
        uc::import_files(
            &db,
            collection_uid,
            params,
            import_config,
            report_progress_fn,
            abort_flag,
        )
        .map_err(transaction_error)
    })?;
    log::info!("Analyzing and optimizing database after import finished");
    analyze_and_optimize_database_stats(&db)?;
    Ok(outcome)
}
