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

use aoide_core_api::collection::Summary;

use super::*;

pub fn load_one(
    connection: &SqliteConnection,
    uid: &EntityUid,
    with_summary: bool,
) -> Result<(Entity, Option<Summary>)> {
    let repo = RepoConnection::new(connection);
    let id = repo.resolve_collection_id(uid)?;
    let (record_hdr, entity) = repo.load_collection_entity(id)?;
    let summary = if with_summary {
        Some(repo.load_collection_summary(record_hdr.id)?)
    } else {
        None
    };
    Ok((entity, summary))
}

pub fn load_all(
    connection: &SqliteConnection,
    kind: Option<&str>,
    with_summary: bool,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<
        Header = RecordHeader,
        Record = (Entity, Option<Summary>),
    >,
) -> Result<()> {
    let repo = RepoConnection::new(connection);
    repo.load_collection_entities(kind, with_summary, pagination, collector)
        .map_err(Into::into)
}

pub fn load_all_kinds(connection: &SqliteConnection) -> Result<Vec<String>> {
    let repo = RepoConnection::new(connection);
    repo.load_all_kinds().map_err(Into::into)
}
