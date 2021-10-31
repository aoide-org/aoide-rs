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

use aoide_core::{entity::EntityUid, media::SourcePath, util::clock::DateTime};

use aoide_repo::{collection::EntityRepo as _, media::source::Repo as _};

use super::*;

pub mod tracker;

pub fn relocate_collected_sources(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    old_path_prefix: &SourcePath,
    new_path_prefix: &SourcePath,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselTransactionError<RepoError>, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let updated_at = DateTime::now_utc();
        Ok(db.relocate_media_sources_by_path_prefix(
            updated_at,
            collection_id,
            old_path_prefix,
            new_path_prefix,
        )?)
    })
    .map_err(Into::into)
}
