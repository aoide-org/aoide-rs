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

use aoide_core::{entity::EntityUid, media::content::ContentPath, util::clock::DateTime};

use aoide_repo::{collection::EntityRepo as _, media::source::CollectionRepo as _};

use super::*;

pub fn relocate(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    old_content_path_prefix: &ContentPath,
    new_content_path_prefix: &ContentPath,
) -> Result<usize> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, RepoTransactionError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        let updated_at = DateTime::now_utc();
        Ok(db.relocate_media_sources_by_content_path_prefix(
            collection_id,
            updated_at,
            old_content_path_prefix,
            new_content_path_prefix,
        )?)
    })
    .map_err(Into::into)
}
