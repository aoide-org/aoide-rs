// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{atomic::AtomicBool, Arc};

use aoide_core::{media::content::ContentPath, track::Track};
use aoide_media_file::io::import::ImportTrackConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use diesel::Connection as _;

use crate::prelude::*;

pub async fn count_sources_in_directories(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::count_sources_in_directories::Params,
) -> Result<Vec<(ContentPath<'static>, usize)>> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::tracker::count_sources_in_directories::count_sources_in_directories(
                    connection,
                    &collection_uid,
                    &params,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn query_status(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::query_status::Params,
) -> Result<aoide_core_api::media::tracker::Status> {
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::tracker::query_status::query_status(
                    connection,
                    &collection_uid,
                    &params,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn scan_directories<P>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::scan_directories::Params,
    report_progress_fn: P,
    abort_flag: Arc<AtomicBool>,
) -> Result<aoide_core_api::media::tracker::scan_directories::Outcome>
where
    P: FnMut(aoide_usecases::media::tracker::scan_directories::ProgressEvent) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let mut report_progress_fn = report_progress_fn;
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::tracker::scan_directories::scan_directories(
                    connection,
                    &collection_uid,
                    &params,
                    &mut report_progress_fn,
                    &abort_flag,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn untrack_directories(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::untrack_directories::Params,
) -> Result<aoide_core_api::media::tracker::untrack_directories::Outcome> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::tracker::untrack_directories::untrack_directories(
                    connection,
                    &collection_uid,
                    &params,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn import_files<InterceptImportedTrackFn, ReportProgressFn>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::import_files::Params,
    import_config: ImportTrackConfig,
    intercept_imported_track_fn: InterceptImportedTrackFn,
    report_progress_fn: ReportProgressFn,
    abort_flag: Arc<AtomicBool>,
) -> Result<aoide_core_api::media::tracker::import_files::Outcome>
where
    InterceptImportedTrackFn: Fn(Track) -> Track + Send + 'static,
    ReportProgressFn:
        FnMut(aoide_usecases::media::tracker::import_files::ProgressEvent) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let mut report_progress_fn = report_progress_fn;
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::tracker::import_files::import_files(
                    connection,
                    &collection_uid,
                    &params,
                    import_config,
                    &intercept_imported_track_fn,
                    &mut report_progress_fn,
                    &abort_flag,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn find_untracked_files<P>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::find_untracked_files::Params,
    report_progress_fn: P,
    abort_flag: Arc<AtomicBool>,
) -> Result<aoide_core_api::media::tracker::find_untracked_files::Outcome>
where
    P: FnMut(aoide_usecases::media::tracker::find_untracked_files::ProgressEvent) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |mut pooled_connection| {
            let mut report_progress_fn = report_progress_fn;
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::tracker::find_untracked_files::visit_directories(
                    connection,
                    &collection_uid,
                    &params,
                    &mut report_progress_fn,
                    &abort_flag,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
