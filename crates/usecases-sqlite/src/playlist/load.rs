// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::playlist::EntityWithEntriesSummary;
use aoide_repo::collection::EntityRepo as _;

use super::*;

pub fn load_entity_with_entries(
    connection: &mut DbConnection,
    entity_uid: &EntityUid,
) -> Result<EntityWithEntries> {
    let mut repo = RepoConnection::new(connection);
    let id = repo.resolve_playlist_id(entity_uid)?;
    repo.load_playlist_entity_with_entries(id)
        .map_err(Into::into)
}

pub fn load_entities_with_entries_summary(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    kind: Option<&str>,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<
        Header = RecordHeader,
        Record = EntityWithEntriesSummary,
    >,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    let collection_id = repo.resolve_collection_id(collection_uid)?;
    repo.load_playlist_entities_with_entries_summary(collection_id, kind, pagination, collector)
        .map_err(Into::into)
}
