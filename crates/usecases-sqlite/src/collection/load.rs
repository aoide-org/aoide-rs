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

use aoide_core_api::collection::{EntityWithSummary, LoadScope};

use super::*;

pub fn load_one(
    connection: &SqliteConnection,
    entity_uid: &EntityUid,
    scope: LoadScope,
) -> Result<EntityWithSummary> {
    let repo = RepoConnection::new(connection);
    let id = repo.resolve_collection_id(entity_uid)?;
    let (record_hdr, entity) = repo.load_collection_entity(id)?;
    let summary = match scope {
        LoadScope::Entity => None,
        LoadScope::EntityWithSummary => Some(repo.load_collection_summary(record_hdr.id)?),
    };
    Ok(EntityWithSummary { entity, summary })
}

pub fn load_all(
    connection: &SqliteConnection,
    kind: Option<&str>,
    scope: LoadScope,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<Header = RecordHeader, Record = EntityWithSummary>,
) -> Result<()> {
    let repo = RepoConnection::new(connection);
    let with_summary = match scope {
        LoadScope::Entity => false,
        LoadScope::EntityWithSummary => true,
    };
    repo.load_collection_entities(kind, with_summary, pagination, collector)
        .map_err(Into::into)
}

pub fn load_all_kinds(connection: &SqliteConnection) -> Result<Vec<String>> {
    let repo = RepoConnection::new(connection);
    repo.load_all_kinds().map_err(Into::into)
}
