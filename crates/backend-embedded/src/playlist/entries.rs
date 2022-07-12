// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use diesel::Connection as _;

use aoide_core::playlist::EntityHeader;
use aoide_core_api::playlist::EntityWithEntriesSummary;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::prelude::*;

pub async fn patch(
    db_gatekeeper: &Gatekeeper,
    entity_header: EntityHeader,
    operations: impl IntoIterator<Item = aoide_usecases_sqlite::playlist::entries::PatchOperation>
        + Send
        + 'static,
) -> Result<EntityWithEntriesSummary> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::playlist::entries::patch(
                    &*pooled_connection,
                    &entity_header,
                    operations,
                )
                .map(|(_, x)| x)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
