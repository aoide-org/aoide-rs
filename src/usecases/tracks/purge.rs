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

use aoide_core::util::url::BaseUrl;

use aoide_repo::collection::EntityRepo as _;

mod uc {
    pub use aoide_core::usecases::media::tracker::import::*;
    pub use aoide_usecases::{
        collection::resolve_collection_id_for_virtual_file_path, tracks::purge::*, Error,
    };
}

use super::*;

pub fn purge_by_media_source_path_predicates(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    path_predicates: Vec<StringPredicate>,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        uc::purge_by_media_source_path_predicates(&db, collection_id, path_predicates)
            .map_err(Into::into)
    })
    .map_err(Into::into)
}

pub fn purge_by_untracked_media_sources(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_url: Option<&BaseUrl>,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (_, source_path_resolver) =
            uc::resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                .map_err(DieselTransactionError::new)?;
        let collection_id = db.resolve_collection_id(collection_uid)?;
        uc::purge_by_untracked_media_sources(&db, collection_id, &source_path_resolver, root_url)
            .map_err(DieselTransactionError::new)
    })
    .map_err(Into::into)
}
