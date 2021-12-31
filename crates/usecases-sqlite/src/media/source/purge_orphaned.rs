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

use aoide_core_api::media::source::purge_orphaned::{Outcome, Params};

use super::*;

mod uc {
    pub use aoide_usecases::media::source::purge_orphaned::purge_orphaned;
}

pub fn purge_orphaned(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &Params,
) -> Result<Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::purge_orphaned(&db, collection_uid, params).map_err(transaction_error)
    })
    .map_err(Into::into)
}
