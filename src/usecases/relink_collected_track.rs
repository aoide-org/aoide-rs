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

use aoide_usecases::relink_collected_track as uc;

pub fn relink_collected_track_by_media_source_uri(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    old_source_uri: &str,
    new_source_uri: &str,
) -> Result<()> {
    let db = RepoConnection::new(connection);
    db.transaction::<_, DieselRepoError, _>(|| {
        Ok(uc::relink_collected_track_by_media_source_uri(
            &db,
            collection_uid,
            old_source_uri,
            new_source_uri,
        )?)
    })
    .map_err(Into::into)
}
