// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::Connection as _;

use aoide_core::playlist::{Entity, EntityHeader, EntityUid, EntityWithEntries, Playlist};
use aoide_core_api::{playlist::EntityWithEntriesSummary, Pagination};
use aoide_repo::{
    playlist::{EntityWithEntriesSummaryCollector, KindFilter, RecordHeader},
    ReservableRecordCollector,
};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use aoide_usecases::playlist::CollectionFilter;

use crate::prelude::*;

pub mod entries;

pub async fn create(
    db_gatekeeper: &Gatekeeper,
    collection_uid: Option<CollectionUid>,
    new_playlist: Playlist,
) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::playlist::create(
                    connection,
                    collection_uid.as_ref(),
                    new_playlist,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn update(
    db_gatekeeper: &Gatekeeper,
    entity_header: EntityHeader,
    modified_playlist: Playlist,
) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::playlist::update(
                    connection,
                    entity_header,
                    modified_playlist,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn purge(db_gatekeeper: &Gatekeeper, entity_uid: EntityUid) -> Result<()> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::playlist::purge(connection, &entity_uid)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

/// Load a single entity including all entries
pub async fn load_one(
    db_gatekeeper: &Gatekeeper,
    entity_uid: EntityUid,
) -> Result<EntityWithEntries> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::playlist::load_one_with_entries(connection, &entity_uid)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

/// Load multiple entities, each with a summary of their entries
pub async fn load_all(
    db_gatekeeper: &Gatekeeper,
    collection_filter: Option<CollectionFilter<'static>>,
    kind_filter: Option<KindFilter<'static>>,
    pagination: Option<Pagination>,
) -> Result<Vec<EntityWithEntriesSummary>> {
    load_all_collecting(
        db_gatekeeper,
        collection_filter,
        kind_filter,
        pagination,
        EntityWithEntriesSummaryCollector::new(Vec::new()),
    )
    .await
    .map(EntityWithEntriesSummaryCollector::finish)
}

/// Load a multiple entities, each with a summary of their entries
pub async fn load_all_collecting<C>(
    db_gatekeeper: &Gatekeeper,
    collection_filter: Option<CollectionFilter<'static>>,
    kind_filter: Option<KindFilter<'static>>,
    pagination: Option<Pagination>,
    collector: C,
) -> Result<C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = EntityWithEntriesSummary>
        + Send
        + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                let mut collector = collector;
                aoide_usecases_sqlite::playlist::load_all_with_entries_summary(
                    connection,
                    collection_filter,
                    kind_filter,
                    pagination.as_ref(),
                    &mut collector,
                )?;
                Ok(collector)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
