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

use aoide_core_ext::media::tracker::purge_untracked_sources::{Outcome, Params};

use super::*;

mod uc {
    pub use aoide_usecases::{
        collection::resolve_collection_id_for_virtual_file_path,
        media::tracker::purge_untracked_sources::*, Error,
    };
}

pub fn purge_untracked_sources(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    params: &Params,
) -> Result<Outcome> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (collection_id, source_path_resolver) =
            uc::resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                .map_err(DieselTransactionError::new)?;
        uc::purge_untracked_sources(&db, &source_path_resolver, collection_id, params)
            .map_err(DieselTransactionError::new)
    })
    .map_err(Into::into)
}
