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

use aoide_core_api::media::tracker::import_files::Params;
use aoide_media::io::import::ImportTrackConfig;

use super::*;

mod uc {
    pub(super) use aoide_core_api::media::tracker::import_files::*;
    pub(super) use aoide_usecases::media::tracker::import_files::*;
}

pub fn import_files<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    params: &Params,
    import_config: ImportTrackConfig,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<uc::Outcome> {
    let repo = RepoConnection::new(connection);
    let outcome = uc::import_files(
        &repo,
        collection_uid,
        params,
        import_config,
        report_progress_fn,
        abort_flag,
    )?;
    Ok(outcome)
}
