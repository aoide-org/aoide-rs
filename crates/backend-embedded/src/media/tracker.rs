// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::Connection as _;

use aoide_media::io::import::ImportTrackConfig;

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::prelude::*;

pub async fn query_status(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::query_status::Params,
) -> Result<aoide_core_api::media::tracker::Status> {
    db_gatekeeper
        .spawn_blocking_read_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::tracker::query_status::query_status(
                    &*pooled_connection,
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
) -> Result<aoide_core_api::media::tracker::scan_directories::Outcome>
where
    P: FnMut(aoide_usecases::media::tracker::scan_directories::ProgressEvent) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, abort_flag| {
            let connection = &*pooled_connection;
            let mut report_progress_fn = report_progress_fn;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::tracker::scan_directories::scan_directories(
                    &*pooled_connection,
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
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::tracker::untrack_directories::untrack_directories(
                    &*pooled_connection,
                    &collection_uid,
                    &params,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn import_files<P>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::tracker::import_files::Params,
    import_config: ImportTrackConfig,
    report_progress_fn: P,
) -> Result<aoide_core_api::media::tracker::import_files::Outcome>
where
    P: FnMut(aoide_usecases::media::tracker::import_files::ProgressEvent) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, abort_flag| {
            let connection = &*pooled_connection;
            let mut report_progress_fn = report_progress_fn;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::tracker::import_files::import_files(
                    &*pooled_connection,
                    &collection_uid,
                    &params,
                    import_config,
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
) -> Result<aoide_core_api::media::tracker::find_untracked_files::Outcome>
where
    P: FnMut(aoide_usecases::media::tracker::find_untracked_files::ProgressEvent) + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |pooled_connection, abort_flag| {
            let connection = &*pooled_connection;
            let mut report_progress_fn = report_progress_fn;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::tracker::find_untracked_files::visit_directories(
                    &*pooled_connection,
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
