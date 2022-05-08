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
