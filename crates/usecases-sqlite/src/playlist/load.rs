// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use aoide_core_api::playlist::EntityWithEntriesSummary;
use aoide_repo::{
    collection::EntityRepo as _,
    playlist::{CollectionFilter as RepoCollectionFilter, KindFilter},
};

use super::*;

pub fn load_one_with_entries(
    connection: &mut DbConnection,
    entity_uid: &EntityUid,
) -> Result<EntityWithEntries> {
    let mut repo = RepoConnection::new(connection);
    let id = repo.resolve_playlist_id(entity_uid)?;
    repo.load_playlist_entity_with_entries(id)
        .map_err(Into::into)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionFilter<'a> {
    pub uid: Option<Cow<'a, CollectionUid>>,
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
    let collection_filter = collection_filter
        .map(|CollectionFilter { uid }| {
            uid.as_ref()
                .map(|uid| repo.resolve_collection_id(uid))
                .transpose()
        })
        .transpose()?
        .map(|id| RepoCollectionFilter { id });
    repo.load_playlist_entities_with_entries_summary(
        collection_filter,
        kind_filter,
        pagination,
        collector,
    )
    .map_err(Into::into)
}
