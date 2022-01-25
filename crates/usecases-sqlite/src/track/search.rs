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

use super::*;

mod uc {
    pub use aoide_core_api::track::search::*;
    pub use aoide_usecases::{track::search::*, Error};
}

pub fn search(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: uc::Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, TransactionError, _>(|| {
        uc::search_with_params(&db, collection_uid, params, pagination, collector)
            .map_err(transaction_error)
    })
    .map_err(Into::into)
}
