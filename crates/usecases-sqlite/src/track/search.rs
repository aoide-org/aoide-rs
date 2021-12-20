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

use aoide_repo::collection::EntityRepo as _;

use super::*;

mod uc {
    pub use aoide_core_ext::track::search::*;
    pub use aoide_usecases::{
        collection::resolve_collection_id_for_virtual_file_path, track::search::*, Error,
    };
}

pub fn search(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    params: uc::Params,
    pagination: &Pagination,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<usize> {
    let db = RepoConnection::new(&pooled_connection);
    db.transaction::<_, TransactionError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        uc::search_with_params(&db, collection_id, params, pagination, collector)
            .map_err(transaction_error)
    })
    .map_err(Into::into)
}
