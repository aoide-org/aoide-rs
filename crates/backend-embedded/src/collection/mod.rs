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

use aoide_core::{
    collection::{Collection, Entity},
    entity::{EntityHeader, EntityUid},
};
use aoide_core_api::{
    collection::{EntityWithSummary, LoadScope},
    Pagination,
};
use aoide_repo::{
    collection::{EntityWithSummaryCollector, RecordHeader},
    prelude::ReservableRecordCollector,
};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{Error, Result};

pub async fn load_all_kinds(db_gatekeeper: &Gatekeeper) -> Result<Vec<String>> {
    db_gatekeeper
        .spawn_blocking_read_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::collection::load::load_all_kinds(&*pooled_connection)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

pub async fn load_all(
    db_gatekeeper: &Gatekeeper,
    kind: Option<String>,
    scope: LoadScope,
    pagination: Option<Pagination>,
) -> Result<Vec<EntityWithSummary>> {
    load_all_with_collector(
        db_gatekeeper,
        kind,
        scope,
        pagination,
        EntityWithSummaryCollector::new(Vec::new()),
    )
    .await
    .map(EntityWithSummaryCollector::finish)
}

pub async fn load_all_with_collector<C>(
    db_gatekeeper: &Gatekeeper,
    kind: Option<String>,
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
        .spawn_blocking_read_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                let mut collector = collector;
                aoide_usecases_sqlite::collection::load::load_all(
                    &*pooled_connection,
                    kind.as_deref(),
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

pub async fn create(db_gatekeeper: &Gatekeeper, new_collection: Collection) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::collection::create::create(
                    &*pooled_connection,
                    new_collection,
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
    modified_collection: Collection,
) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::collection::update::update(
                    &*pooled_connection,
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
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::collection::purge::purge(&*pooled_connection, &entity_uid)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
