// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, atomic::AtomicBool};

use diesel::Connection as _;

use aoide_core::{
    media::content::ContentPath,
    track::{Entity, EntityUid, Track},
};
use aoide_core_api::{Pagination, track::find_unsynchronized::UnsynchronizedTrackEntity};
use aoide_repo::{RecordCollector, ReservableRecordCollector, track::RecordHeader};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::prelude::*;

pub mod vfs;

#[derive(Debug, Default)]
pub struct EntityCollector(Vec<Entity>);

impl EntityCollector {
    #[must_use]
    pub const fn new(inner: Vec<Entity>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn finish(self) -> Vec<Entity> {
        let Self(inner) = self;
        inner
    }
}

impl From<EntityCollector> for Vec<Entity> {
    fn from(from: EntityCollector) -> Self {
        let EntityCollector(inner) = from;
        inner
    }
}

impl RecordCollector for EntityCollector {
    type Header = RecordHeader;
    type Record = Entity;

    fn collect(&mut self, _record_header: RecordHeader, entity: Entity) {
        let Self(inner) = self;
        inner.push(entity);
    }
}

impl ReservableRecordCollector for EntityCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}

pub async fn load_one(db_gatekeeper: &Gatekeeper, entity_uid: EntityUid) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::track::load::load_one(connection, &entity_uid)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn load_many_collecting<I, C>(
    db_gatekeeper: &Gatekeeper,
    entity_uids: I,
    collector: C,
) -> Result<C>
where
    I: IntoIterator<Item = EntityUid> + Send + 'static,
    C: ReservableRecordCollector<Header = RecordHeader, Record = Entity> + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                let mut collector = collector;
                aoide_usecases_sqlite::track::load::load_many(
                    connection,
                    entity_uids,
                    &mut collector,
                )?;
                Ok(collector)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn load_many<I>(db_gatekeeper: &Gatekeeper, entity_uids: I) -> Result<Vec<Entity>>
where
    I: IntoIterator<Item = EntityUid> + Send + 'static,
{
    load_many_collecting(db_gatekeeper, entity_uids, EntityCollector::new(Vec::new()))
        .await
        .map(EntityCollector::finish)
}

pub async fn search(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::track::search::Params,
    pagination: Pagination,
) -> Result<Vec<Entity>> {
    search_collecting(
        db_gatekeeper,
        collection_uid,
        params,
        pagination,
        EntityCollector::new(Vec::new()),
    )
    .await
    .map(EntityCollector::finish)
}

pub async fn search_collecting<C>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::track::search::Params,
    pagination: Pagination,
    collector: C,
) -> Result<C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = Entity> + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                let mut collector = collector;
                aoide_usecases_sqlite::track::search::search(
                    connection,
                    &collection_uid,
                    &params,
                    &pagination,
                    &mut collector,
                )?;
                Ok(collector)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn find_unsynchronized(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::track::find_unsynchronized::Params,
    pagination: Pagination,
) -> Result<Vec<UnsynchronizedTrackEntity>> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::track::find_unsynchronized::find_unsynchronized(
                    connection,
                    &collection_uid,
                    params,
                    &pagination,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn replace_many_by_media_source_content_path<I>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_usecases::track::replace::Params,
    validated_track_iter: I,
) -> Result<aoide_core_api::track::replace::Summary>
where
    I: IntoIterator<Item = aoide_usecases::track::ValidatedInput> + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::track::replace::replace_many_by_media_source_content_path(
                    connection,
                    &collection_uid,
                    &params,
                    validated_track_iter,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn import_and_replace_many_by_local_file_path<ContentPathIter, InterceptImportedTrackFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_usecases::track::import_and_replace::Params,
    content_path_iter: ContentPathIter,
    expected_content_path_count: Option<usize>,
    intercept_imported_track_fn: InterceptImportedTrackFn,
    abort_flag: Arc<AtomicBool>,
) -> Result<aoide_usecases::track::import_and_replace::Outcome>
where
    ContentPathIter: IntoIterator<Item = ContentPath<'static>> + Send + 'static,
    InterceptImportedTrackFn: Fn(Track) -> Track + Send + 'static,
{
    db_gatekeeper
    .spawn_blocking_write_task(move |mut pooled_connection| {
        let connection = &mut *pooled_connection;
        connection.transaction::<_, Error, _>(|connection| {
        aoide_usecases_sqlite::track::import_and_replace::import_and_replace_many_by_local_file_path(
            connection,
            &collection_uid,
            content_path_iter,
            expected_content_path_count,
            &params,
            &intercept_imported_track_fn,
            &abort_flag,
        )
        })
    })
    .await
    .map_err(Into::into)
    .unwrap_or_else(Err)
}
