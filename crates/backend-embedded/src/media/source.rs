// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::media::content::ContentPath;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use diesel::Connection as _;

use crate::prelude::*;

pub async fn purge_orphaned(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::source::purge_orphaned::Params,
) -> Result<aoide_core_api::media::source::purge_orphaned::Outcome> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::source::purge_orphaned::purge_orphaned(
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

pub async fn purge_untracked(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    params: aoide_core_api::media::source::purge_untracked::Params,
) -> Result<aoide_core_api::media::source::purge_untracked::Outcome> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::source::purge_untracked::purge_untracked(
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

pub async fn relocate(
    db_gatekeeper: &Gatekeeper,
    collection_uid: CollectionUid,
    old_path_prefix: ContentPath<'static>,
    new_path_prefix: ContentPath<'static>,
) -> Result<usize> {
    db_gatekeeper
        .spawn_blocking_write_task(move |mut pooled_connection| {
            let connection = &mut *pooled_connection;
            connection.transaction::<_, Error, _>(|connection| {
                aoide_usecases_sqlite::media::source::relocate::relocate(
                    connection,
                    &collection_uid,
                    &old_path_prefix,
                    &new_path_prefix,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
