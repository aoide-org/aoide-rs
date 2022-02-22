// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use diesel::Connection as _;

use aoide_core::{entity::EntityUid, media::content::ContentPath};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{Error, Result};

pub async fn purge_orphaned(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    params: aoide_core_api::media::source::purge_orphaned::Params,
) -> Result<aoide_core_api::media::source::purge_orphaned::Outcome> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::source::purge_orphaned::purge_orphaned(
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

pub async fn purge_untracked(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    params: aoide_core_api::media::source::purge_untracked::Params,
) -> Result<aoide_core_api::media::source::purge_untracked::Outcome> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::source::purge_untracked::purge_untracked(
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

pub async fn relocate(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    old_path_prefix: ContentPath,
    new_path_prefix: ContentPath,
) -> Result<usize> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::media::source::relocate::relocate(
                    &*pooled_connection,
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
