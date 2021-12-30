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

use aoide_core::entity::EntityUid;

use aoide_core_api::media::tracker::{query_status::Params, Status};

use super::*;

mod uc {
    pub use aoide_usecases::{
        media::{
            tracker::{query_status::*, *},
            *,
        },
        Error,
    };
}

pub fn query_status(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &Params,
) -> Result<Status> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::query_status(&db, collection_uid, params).map_err(transaction_error)
    })
    .map_err(Into::into)
}
