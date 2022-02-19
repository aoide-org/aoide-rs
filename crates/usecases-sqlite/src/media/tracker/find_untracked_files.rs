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

use aoide_core::entity::EntityUid;
use aoide_core_api::media::tracker::{find_untracked_files::Outcome, FsTraversalParams};

use super::*;

mod uc {
    pub use aoide_core_api::media::tracker::find_untracked_files::*;
    pub use aoide_usecases::{media::tracker::find_untracked_files::*, Error};
}

pub fn visit_directories<ReportProgressFn: FnMut(uc::ProgressEvent)>(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &FsTraversalParams,
    report_progress_fn: &mut ReportProgressFn,
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let repo = RepoConnection::new(connection);
    uc::visit_directories(
        &repo,
        collection_uid,
        params,
        report_progress_fn,
        abort_flag,
    )
    .map_err(Into::into)
}
