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

use super::*;

pub fn load_one(connection: &SqliteConnection, uid: &EntityUid) -> Result<Entity> {
    let repo = RepoConnection::new(connection);
    let (_, entity) = repo.load_track_entity_by_uid(uid)?;
    Ok(entity)
}

pub fn load_many(
    connection: &SqliteConnection,
    uids: impl IntoIterator<Item = EntityUid>,
    collector: &mut impl RecordCollector<Header = RecordHeader, Record = Entity>,
) -> Result<()> {
    let repo = RepoConnection::new(connection);
    for uid in uids {
        match repo.load_track_entity_by_uid(&uid) {
            Ok((record_header, entity)) => {
                collector.collect(record_header, entity);
            }
            Err(RepoError::NotFound) => {
                log::debug!("Track with UID '{}' not found", uid);
                continue;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }
    Ok(())
}
