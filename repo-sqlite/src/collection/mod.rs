// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
pub(crate) mod schema;

use self::{models::*, schema::*};

use crate::{track::schema::tbl_track, util::*};

use aoide_core::{
    collection::{track::ItemBody as TrackItemBody, *},
    entity::{EntityHeader, EntityRevision, EntityUid},
};

use aoide_repo::{collection::*, track::Repo as _, *};

impl<'db> Repo for crate::Connection<'db> {
    fn resolve_collection_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>> {
        tbl_collection::table
            .select(tbl_collection::id)
            .filter(tbl_collection::uid.eq(uid.as_ref()))
            .first::<RepoId>(self.as_ref())
            .optional()
            .map_err(Into::into)
    }

    fn insert_collection(&self, entity: &Entity) -> RepoResult<()> {
        let insertable = InsertableEntity::bind(entity);
        let query = diesel::insert_into(tbl_collection::table).values(&insertable);
        query.execute(self.as_ref())?;
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
                    .and(tbl_collection::rev_no.eq(prev_rev.no as i64))
                    .and(tbl_collection::rev_ts.eq((prev_rev.ts.0).0)),
            );
            let query = diesel::update(target).set(&updatable);
            let rows_affected: usize = query.execute(self.as_ref())?;
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
        let rows_affected: usize = query.execute(self.as_ref())?;
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
            .first::<QueryableEntity>(self.as_ref())
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
            .load::<QueryableEntity>(self.as_ref())
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn find_collections_by_name(&self, name: &str) -> RepoResult<Vec<Entity>> {
        tbl_collection::table
            .filter(tbl_collection::name.eq(name))
            .load::<QueryableEntity>(self.as_ref())
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
            .load::<QueryableEntity>(self.as_ref())
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
            .load::<QueryableEntity>(self.as_ref())
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }
}

impl<'db> TrackEntryRepo for crate::Connection<'db> {
    fn replace_track_entry(
        &self,
        collection_uid: &EntityUid,
        track_uid: &EntityUid,
        entry: SingleTrackEntry,
    ) -> RepoResult<()> {
        let SingleTrackEntry {
            added_at,
            item: item_body,
        } = entry;
        let collection_id =
            self.resolve_collection_id(collection_uid)
                .and_then(|collection_id| {
                    collection_id.ok_or_else(|| anyhow!("collection {} not found", collection_uid))
                })?;
        let track_id = self.resolve_track_id(&track_uid).and_then(|track_id| {
            track_id.ok_or_else(|| anyhow!("track {} not found", track_uid))
        })?;
        let updatable = UpdatableCollectionTrack::bind(&item_body);
        let update_query = diesel::update(tbl_collection_track::table).set(&updatable);
        let rows_updated: usize = update_query.execute(self.as_ref())?;
        debug_assert!(rows_updated <= 1);
        if rows_updated > 0 {
            return Ok(());
        }
        let insertable =
            InsertableCollectionTrack::bind(collection_id, track_id, added_at, &item_body);
        let query = diesel::insert_into(tbl_collection_track::table).values(&insertable);
        query.execute(self.as_ref())?;
        Ok(())
    }

    fn remove_track_entry(
        &self,
        collection_uid: &EntityUid,
        track_uid: &EntityUid,
    ) -> RepoResult<bool> {
        let target = tbl_collection_track::table
            .filter(
                tbl_collection_track::collection_id.eq_any(
                    tbl_collection::table
                        .select(tbl_collection::id)
                        .filter(tbl_collection::uid.eq(collection_uid.as_ref())),
                ),
            )
            .filter(
                tbl_collection_track::track_id.eq_any(
                    tbl_track::table
                        .select(tbl_track::id)
                        .filter(tbl_track::uid.eq(track_uid.as_ref())),
                ),
            );
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref())?;
        debug_assert!(rows_affected <= 1);
        Ok(rows_affected > 0)
    }

    fn remove_all_track_entries(&self, collection_uid: &EntityUid) -> RepoResult<usize> {
        let target = tbl_collection_track::table.filter(
            tbl_collection_track::collection_id.eq_any(
                tbl_collection::table
                    .select(tbl_collection::id)
                    .filter(tbl_collection::uid.eq(collection_uid.as_ref())),
            ),
        );
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref())?;
        Ok(rows_affected)
    }

    fn load_track_entry(
        &self,
        collection_uid: &EntityUid,
        track_uid: &EntityUid,
    ) -> RepoResult<Option<SingleTrackEntry>> {
        tbl_collection_track::table
            .filter(
                tbl_collection_track::collection_id.eq_any(
                    tbl_collection::table
                        .select(tbl_collection::id)
                        .filter(tbl_collection::uid.eq(collection_uid.as_ref())),
                ),
            )
            .filter(
                tbl_collection_track::track_id.eq_any(
                    tbl_track::table
                        .select(tbl_track::id)
                        .filter(tbl_track::uid.eq(track_uid.as_ref())),
                ),
            )
            .first::<QueryableCollectionTrack>(self.as_ref())
            .optional()
            .map(|o| o.map(Into::into))
            .map_err(Into::into)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
