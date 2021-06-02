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

use super::*;

use aoide_core::{entity::EntityUid, util::url::BaseUrl};

mod uc {
    pub use aoide_usecases::{
        collection::resolve_collection_id_for_virtual_file_path,
        media::{
            tracker::{query_status::*, *},
            *,
        },
        Error,
    };
}

pub use aoide_core::usecases::media::tracker::Status;

pub fn query_status(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_url: Option<&BaseUrl>,
) -> Result<Status> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
        let (collection_id, source_path_resolver) =
            uc::resolve_collection_id_for_virtual_file_path(&db, collection_uid, None)
                .map_err(DieselTransactionError::new)?;
        uc::query_status(&db, collection_id, &source_path_resolver, root_url)
            .map_err(DieselTransactionError::new)
    })
    .map_err(Into::into)
}
