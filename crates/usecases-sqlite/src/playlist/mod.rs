// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    playlist::EntityWithEntries, CollectionUid, Playlist, PlaylistEntity, PlaylistHeader,
    PlaylistUid,
};
use aoide_core_api::{playlist::EntityWithEntriesSummary, Pagination};
use aoide_repo::{
    playlist::{EntityRepo as _, KindFilter, RecordHeader},
    ReservableRecordCollector,
};
use aoide_repo_sqlite::DbConnection;
use aoide_usecases as uc;

use crate::{RepoConnection, Result};

pub mod entries;

pub fn create(
    connection: &mut DbConnection,
    collection_uid: Option<&CollectionUid>,
    new_playlist: Playlist,
) -> Result<PlaylistEntity> {
    let created_entity = uc::playlist::create_entity(new_playlist)?;
    let mut repo = RepoConnection::new(connection);
    uc::playlist::store_created_entity(&mut repo, collection_uid, &created_entity)?;
    Ok(created_entity)
}

pub fn update(
    connection: &mut DbConnection,
    entity_header: PlaylistHeader,
    modified_playlist: Playlist,
) -> Result<PlaylistEntity> {
    let updated_entity = uc::playlist::update_entity(entity_header, modified_playlist)?;
    let mut repo = RepoConnection::new(connection);
    uc::playlist::store_updated_entity(&mut repo, &updated_entity)?;
    Ok(updated_entity)
}

pub fn purge(connection: &mut DbConnection, entity_uid: &PlaylistUid) -> Result<()> {
    let mut repo = RepoConnection::new(connection);
    let id = repo.resolve_playlist_id(entity_uid)?;
    repo.purge_playlist_entity(id).map_err(Into::into)
}

pub fn load_one_with_entries(
    connection: &mut DbConnection,
    entity_uid: &PlaylistUid,
) -> Result<EntityWithEntries> {
    let mut repo = RepoConnection::new(connection);
    uc::playlist::load_one_with_entries(&mut repo, entity_uid).map_err(Into::into)
}

pub fn load_all_with_entries_summary(
    connection: &mut DbConnection,
    collection_filter: Option<uc::playlist::CollectionFilter<'_>>,
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

pub fn patch_entries(
    connection: &mut DbConnection,
    entity_header: &PlaylistHeader,
    operations: impl IntoIterator<Item = uc::playlist::entries::PatchOperation>,
) -> Result<(RecordHeader, EntityWithEntriesSummary)> {
    let mut repo = RepoConnection::new(connection);
    uc::playlist::entries::patch(&mut repo, entity_header, operations).map_err(Into::into)
}
