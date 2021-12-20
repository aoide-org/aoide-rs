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
use aoide_core_ext::media::tracker::{find_untracked_files::Outcome, FsTraversalParams};
use aoide_usecases::media::tracker::find_untracked_files::ProgressEvent;

use super::*;

mod uc {
    pub use aoide_core_ext::media::tracker::find_untracked_files::*;
    pub use aoide_usecases::{media::tracker::find_untracked_files::*, Error};
}

pub fn visit_directories(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &FsTraversalParams,
    progress_event_fn: &mut impl FnMut(ProgressEvent),
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::visit_directories(&db, collection_uid, params, progress_event_fn, abort_flag)
            .map_err(transaction_error)
    })
    .map_err(Into::into)
}
