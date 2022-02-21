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
    entity::{EntityHeader, EntityUid},
    playlist::{Entity, EntityWithEntries, Playlist},
};
use aoide_core_api::{playlist::EntityWithEntriesSummary, Pagination};
use aoide_repo::{
    playlist::{EntityWithEntriesSummaryCollector, RecordHeader},
    prelude::ReservableRecordCollector,
};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{Error, Result};

/// Load a single entity including all entries
pub async fn load_one(
    db_gatekeeper: &Gatekeeper,
    entity_uid: EntityUid,
) -> Result<EntityWithEntries> {
    db_gatekeeper
        .spawn_blocking_read_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::playlist::load::load_entity_with_entries(
                    &*pooled_connection,
                    &entity_uid,
                )
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}

/// Load a multiple entities, each with a summary of their entries
pub async fn load_all(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    kind: Option<String>,
    pagination: Option<Pagination>,
) -> Result<Vec<EntityWithEntriesSummary>> {
    load_all_collecting(
        db_gatekeeper,
        collection_uid,
        kind,
        pagination,
        EntityWithEntriesSummaryCollector::new(Vec::new()),
    )
    .await
    .map(EntityWithEntriesSummaryCollector::finish)
}

/// Load a multiple entities, each with a summary of their entries
pub async fn load_all_collecting<C>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    kind: Option<String>,
    pagination: Option<Pagination>,
    collector: C,
) -> Result<C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = EntityWithEntriesSummary>
        + Send
        + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                let mut collector = collector;
                aoide_usecases_sqlite::playlist::load::load_entities_with_entries_summary(
                    &*pooled_connection,
                    &collection_uid,
                    kind.as_deref(),
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

pub async fn create(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    new_playlist: Playlist,
) -> Result<Entity> {
    db_gatekeeper
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::playlist::create::create(
                    &*pooled_connection,
                    &collection_uid,
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
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::playlist::update::update(
                    &*pooled_connection,
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
        .spawn_blocking_write_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                aoide_usecases_sqlite::playlist::purge::purge(&*pooled_connection, &entity_uid)
            })
        })
        .await
        .map_err(Into::into)
        .unwrap_or_else(Err)
}
