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

use aoide_usecases::track::resolve as uc;

use super::*;

pub fn resolve_by_media_source_content_paths(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    media_source_paths: Vec<String>,
) -> Result<Vec<(String, EntityHeader)>> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, RepoTransactionError, _>(|| {
        let collection_id = db.resolve_collection_id(collection_uid)?;
        uc::resolve_by_media_source_content_paths(&db, collection_id, media_source_paths)
            .map_err(Into::into)
    })
    .map_err(Into::into)
}
