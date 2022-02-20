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

pub fn load_entity_with_entries(
    connection: &SqliteConnection,
    entity_uid: &EntityUid,
) -> Result<EntityWithEntries> {
    let repo = RepoConnection::new(connection);
    let id = repo.resolve_playlist_id(entity_uid)?;
    repo.load_playlist_entity_with_entries(id)
        .map_err(Into::into)
}

pub fn load_entities_with_entries_summary(
    connection: &SqliteConnection,
    collection_uid: &EntityUid,
    kind: Option<&str>,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<
        Header = RecordHeader,
        Record = (Entity, EntriesSummary),
    >,
) -> Result<()> {
    let repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    repo.load_playlist_entities_with_entries_summary(collection_id, kind, pagination, collector)
        .map_err(Into::into)
}
