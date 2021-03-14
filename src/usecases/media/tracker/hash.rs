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

use aoide_core::entity::EntityUid;

use aoide_media::fs::digest;

mod uc {
    pub use aoide_usecases::{media::tracker::hash::*, Error};
}

use aoide_usecases::collection::resolve_virtual_file_path_collection_id;
use digest::ProgressEvent;
use std::sync::atomic::AtomicBool;
use url::Url;

pub use uc::{Outcome, Summary};

pub fn hash_directories_recursively(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    root_dir_url: &Url,
    max_depth: Option<usize>,
    progress_fn: &mut impl FnMut(&ProgressEvent),
    abort_flag: &AtomicBool,
) -> Result<Outcome> {
    let db = RepoConnection::new(connection);
    Ok(
        db.transaction::<_, DieselTransactionError<uc::Error>, _>(|| {
            let (collection_id, source_path_resolver) =
                resolve_virtual_file_path_collection_id(&db, collection_uid)
                    .map_err(DieselTransactionError::new)?;
            Ok(uc::hash_directories_recursively(
                &db,
                collection_id,
                root_dir_url,
                &source_path_resolver,
                max_depth,
                progress_fn,
                abort_flag,
            )
            .map_err(DieselTransactionError::new)?)
        })?,
    )
}
