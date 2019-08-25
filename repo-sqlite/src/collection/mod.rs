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

use crate::util::*;

use aoide_core::{
    collection::*,
    entity::{EntityHeader, EntityRevision, EntityUid},
};

use aoide_repo::{collection::*, entity::Repo as EntityRepo, *};

///////////////////////////////////////////////////////////////////////
// Repository
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Repository<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> Repository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }
}

impl<'a> EntityRepo for Repository<'a> {
    fn resolve_repo_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>> {
        tbl_collection::table
            .select(tbl_collection::id)
            .filter(tbl_collection::uid.eq(uid.as_ref()))
            .first::<RepoId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}

impl<'a> Repo for Repository<'a> {
    fn insert_collection(&self, entity: &Entity) -> RepoResult<()> {
        let insertable = InsertableEntity::bind(entity);
        let query = diesel::insert_into(tbl_collection::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn update_collection(
        &self,
        entity: &Entity,
    ) -> RepoResult<(EntityRevision, Option<EntityRevision>)> {
        let prev_rev = entity.hdr.rev;
        let next_rev = prev_rev.next();
        {
            let updatable = UpdatableEntity::bind(&next_rev, &entity.body);
            let target = tbl_collection::table.filter(
                tbl_collection::uid
                    .eq(entity.hdr.uid.as_ref())
                    .and(tbl_collection::rev_ver.eq(prev_rev.ver as i64))
                    .and(tbl_collection::rev_ts.eq((prev_rev.ts.0).0)),
            );
            let query = diesel::update(target).set(&updatable);
            let rows_affected: usize = query.execute(self.connection)?;
            debug_assert!(rows_affected <= 1);
            if rows_affected < 1 {
                return Ok((prev_rev, None));
            }
        }
        Ok((prev_rev, Some(next_rev)))
    }

    fn delete_collection(&self, uid: &EntityUid) -> RepoResult<Option<()>> {
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

    fn load_collection(&self, uid: &EntityUid) -> RepoResult<Option<Entity>> {
        tbl_collection::table
            .filter(tbl_collection::uid.eq(uid.as_ref()))
            .first::<QueryableEntity>(self.connection)
            .optional()
            .map(|o| o.map(Into::into))
            .map_err(Into::into)
    }

    fn list_collections(&self, pagination: Pagination) -> RepoResult<Vec<Entity>> {
        let mut target = tbl_collection::table
            .then_order_by(tbl_collection::rev_ts.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableEntity>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_collections_by_name(&self, name: &str) -> RepoResult<Vec<Entity>> {
        tbl_collection::table
            .filter(tbl_collection::name.eq(name))
            .load::<QueryableEntity>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_collections_by_name_starting_with(
        &self,
        name_prefix: &str,
        pagination: Pagination,
    ) -> RepoResult<Vec<Entity>> {
        let mut target = tbl_collection::table
            .filter(tbl_collection::name.like(format!("{}%", name_prefix)))
            .then_order_by(tbl_collection::rev_ts.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableEntity>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_collections_by_name_containing(
        &self,
        partial_name: &str,
        pagination: Pagination,
    ) -> RepoResult<Vec<Entity>> {
        let mut target = tbl_collection::table
            .filter(tbl_collection::name.like(format!("%{}%", partial_name)))
            .then_order_by(tbl_collection::rev_ts.desc())
            .into_boxed();

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableEntity>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
