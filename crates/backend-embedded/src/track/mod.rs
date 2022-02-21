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

use aoide_core::{entity::EntityUid, track::Entity};
use aoide_core_api::Pagination;
use aoide_repo::{
    prelude::{RecordCollector, ReservableRecordCollector},
    track::RecordHeader,
};
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{Error, Result};

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

pub async fn search(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    params: aoide_core_api::track::search::Params,
    pagination: Pagination,
) -> Result<Vec<Entity>> {
    search_with_collector(
        db_gatekeeper,
        collection_uid,
        params,
        pagination,
        EntityCollector::new(Vec::new()),
    )
    .await
    .map(EntityCollector::finish)
}

pub async fn search_with_collector<C>(
    db_gatekeeper: &Gatekeeper,
    collection_uid: EntityUid,
    params: aoide_core_api::track::search::Params,
    pagination: Pagination,
    collector: C,
) -> Result<C>
where
    C: ReservableRecordCollector<Header = RecordHeader, Record = Entity> + Send + 'static,
{
    db_gatekeeper
        .spawn_blocking_read_task(move |pooled_connection, _abort_flag| {
            let connection = &*pooled_connection;
            connection.transaction::<_, Error, _>(|| {
                let mut collector = collector;
                aoide_usecases_sqlite::track::search::search(
                    &*pooled_connection,
                    &collection_uid,
                    params,
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
