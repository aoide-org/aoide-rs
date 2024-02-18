// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, Entity, EntityHeader, EntityUid};
use aoide_core_api::{
    collection::{EntityWithSummary, LoadScope},
    Pagination,
};
use aoide_repo::{
    collection::{EntityWithSummaryCollector, KindFilter, MediaSourceRootUrlFilter, RecordHeader},
    prelude::{RepoError, ReservableRecordCollector},
};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use diesel::Connection as _;

use crate::prelude::*;

pub async fn load_all_kinds(db_gatekeeper: &Gatekeeper) -> Result<Vec<String>> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::collection::load_all_kinds(connection)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn load_all(
    db_gatekeeper: &Gatekeeper,
    kind_filter: Option<KindFilter<'static>>,
    media_source_root_url: Option<MediaSourceRootUrlFilter>,
    scope: LoadScope,
    pagination: Option<Pagination>,
) -> Result<Vec<EntityWithSummary>> {
    load_all_collecting(
        db_gatekeeper,
        kind_filter,
        media_source_root_url,
        scope,
        pagination,
        EntityWithSummaryCollector::new(Vec::new()),
    )
    .await
    .map(EntityWithSummaryCollector::finish)
}

pub async fn load_all_collecting<C>(
    db_gatekeeper: &Gatekeeper,
    kind_filter: Option<KindFilter<'static>>,
    media_source_root_url: Option<MediaSourceRootUrlFilter>,
    scope: LoadScope,
    pagination: Option<Pagination>,
    collector: C,
) -> Result<C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = EntityWithSummary>
        + Send
        + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                let mut collector = collector;
                aoide_usecases_sqlite::collection::load_all(
                    connection,
                    kind_filter,
                    media_source_root_url.as_ref(),
                    scope,
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

pub async fn load_one(
    db_gatekeeper: &Gatekeeper,
    entity_uid: EntityUid,
    load_scope: LoadScope,
) -> Result<EntityWithSummary> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::collection::load_one(connection, &entity_uid, load_scope)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn try_load_one(
    db_gatekeeper: &Gatekeeper,
    entity_uid: EntityUid,
    load_scope: LoadScope,
) -> Result<Option<EntityWithSummary>> {
    match load_one(db_gatekeeper, entity_uid.clone(), load_scope).await {
        Ok(entity_with_summary) => Ok(Some(entity_with_summary)),
        Err(Error::Repository(RepoError::NotFound)) => Ok(None),
        Err(err) => Err(err),
    }
}

pub async fn create(db_gatekeeper: &Gatekeeper, new_collection: Collection) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::collection::create(connection, new_collection)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn update(
    db_gatekeeper: &Gatekeeper,
    entity_header: EntityHeader,
    modified_collection: Collection,
) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::collection::update(
                    connection,
                    entity_header,
                    modified_collection,
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
                aoide_usecases_sqlite::collection::purge(connection, &entity_uid)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
