// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::playlist::EntityWithEntriesSummary;
use aoide_repo::playlist::KindFilter;
use uc::playlist::CollectionFilter;

use super::*;

pub fn load_one_with_entries(
    connection: &mut DbConnection,
    entity_uid: &EntityUid,
) -> Result<EntityWithEntries> {
    let mut repo = RepoConnection::new(connection);
    uc::playlist::load_one_with_entries(&mut repo, entity_uid).map_err(Into::into)
}

pub fn load_all_with_entries_summary(
    connection: &mut DbConnection,
    collection_filter: Option<CollectionFilter<'_>>,
    kind_filter: Option<KindFilter<'_>>,
    pagination: Option<&Pagination>,
    collector: &mut impl ReservableRecordCollector<
        Header = RecordHeader,
        Record = EntityWithEntriesSummary,
    >,
) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    uc::playlist::load_all_with_entries_summary(
        &mut repo,
        collection_filter,
        kind_filter,
        pagination,
        collector,
    )
    .map_err(Into::into)
}
