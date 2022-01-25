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

use aoide_repo::collection::EntityRepo as _;

use super::*;

mod uc {
    pub use aoide_usecases::{track::purge::*, Error};
}

pub fn purge_by_media_source_path_predicates(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    path_predicates: Vec<StringPredicate>,
) -> Result<uc::PurgeByMediaSourcePathPredicatesSummary> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, RepoTransactionError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        uc::purge_by_media_source_path_predicates(&db, collection_id, path_predicates)
            .map_err(Into::into)
    })
    .map_err(Into::into)
}
