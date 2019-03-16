// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

mod models;
pub mod schema;

use self::{models::*, schema::*};

use crate::{
    api::{collection::*, entity::*, Pagination},
    storage::util::*,
};

///////////////////////////////////////////////////////////////////////
/// CollectionRepository
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub struct CollectionRepository<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> CollectionRepository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }
}

impl<'a> EntityStorage for CollectionRepository<'a> {
    fn find_storage_id(&self, uid: &EntityUid) -> EntityStorageResult<Option<StorageId>> {
        tbl_collection::table
            .select(tbl_collection::id)
            .filter(tbl_collection::uid.eq(uid.as_ref()))
            .first::<StorageId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}

impl<'a> Collections for CollectionRepository<'a> {
    fn create_entity(&self, body: Collection) -> CollectionsResult<CollectionEntity> {
        let entity = CollectionEntity::new(EntityHeader::initial(), body);
        self.insert_entity(&entity).and(Ok(entity))
    }

    fn insert_entity(&self, entity: &CollectionEntity) -> CollectionsResult<()> {
        let insertable = InsertableCollectionsEntity::bind(entity);
        let query = diesel::insert_into(tbl_collection::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn update_entity(
        &self,
        entity: &CollectionEntity,
    ) -> CollectionsResult<(EntityRevision, Option<EntityRevision>)> {
        let prev_revision = *entity.header().revision();
        let next_revision = prev_revision.next();
        {
            let updatable = UpdatableCollectionsEntity::bind(&next_revision, &entity.body());
            let target = tbl_collection::table.filter(
                tbl_collection::uid
                    .eq(entity.header().uid().as_ref())
                    .and(tbl_collection::rev_no.eq(prev_revision.ordinal() as i64))
                    .and(tbl_collection::rev_ts.eq((prev_revision.instant().0).0)),
            );
            let query = diesel::update(target).set(&updatable);
            let rows_affected: usize = query.execute(self.connection)?;
            debug_assert!(rows_affected <= 1);
            if rows_affected < 1 {
                return Ok((prev_revision, None));
            }
        }
        Ok((prev_revision, Some(next_revision)))
    }

    fn delete_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<()>> {
        let target = tbl_collection::table.filter(tbl_collection::uid.eq(uid.as_ref()));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.connection)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }

    fn load_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>> {
        tbl_collection::table
            .filter(tbl_collection::uid.eq(uid.as_ref()))
            .first::<QueryableCollectionsEntity>(self.connection)
            .optional()
            .map(|o| o.map(|o| o.into()))
            .map_err(Into::into)
    }

    fn list_entities(&self, pagination: Pagination) -> CollectionsResult<Vec<CollectionEntity>> {
        let mut target = tbl_collection::table
            .then_order_by(tbl_collection::rev_ts.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableCollectionsEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(Into::into)
    }

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>> {
        tbl_collection::table
            .filter(tbl_collection::name.eq(name))
            .load::<QueryableCollectionsEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(Into::into)
    }

    fn find_entities_by_name_starting_with(
        &self,
        name_prefix: &str,
        pagination: Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let mut target = tbl_collection::table
            .filter(tbl_collection::name.like(format!("{}%", name_prefix)))
            .then_order_by(tbl_collection::rev_ts.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableCollectionsEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(Into::into)
    }

    fn find_entities_by_name_containing(
        &self,
        partial_name: &str,
        pagination: Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>> {
        let mut target = tbl_collection::table
            .filter(tbl_collection::name.like(format!("%{}%", partial_name)))
            .then_order_by(tbl_collection::rev_ts.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableCollectionsEntity>(self.connection)
            .map(|v| v.into_iter().map(|r| r.into()).collect())
            .map_err(Into::into)
    }
}

///////////////////////////////////////////////////////////////////////
/// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
