// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
mod schema;
pub mod util;

use self::{models::*, schema::*, util::RepositoryHelper};

use crate::util::*;

use aoide_core::{
    entity::{
        EntityHeader, EntityRevision, EntityRevisionUpdateResult, EntityUid, EntityVersionNumber,
    },
    playlist::*,
    util::clock::{TickInstant, TickType, Ticks},
};

use aoide_repo::{
    entity::{EntityBodyData, EntityData, Repo as EntityRepo},
    playlist::*,
    *,
};

use bigdecimal::BigDecimal;
use diesel::dsl;
use num_bigint::{Sign, ToBigInt};
use num_traits::cast::ToPrimitive;

///////////////////////////////////////////////////////////////////////
// Repository
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Repository<'a> {
    connection: &'a diesel::SqliteConnection,
    helper: RepositoryHelper<'a>,
}

impl<'a> Repository<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self {
            connection,
            helper: RepositoryHelper::new(connection),
        }
    }
}

impl<'a> EntityRepo for Repository<'a> {
    fn resolve_repo_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>> {
        tbl_playlist::table
            .select(tbl_playlist::id)
            .filter(tbl_playlist::uid.eq(uid.as_ref()))
            .first::<RepoId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}

impl<'a> Repo for Repository<'a> {
    fn insert_playlist(&self, entity: &Entity, body_data: EntityBodyData) -> RepoResult<()> {
        {
            let (data_fmt, data_ver, data_blob) = body_data;
            let insertable = InsertableEntity::bind(&entity.hdr, data_fmt, data_ver, &data_blob);
            let query = diesel::insert_into(tbl_playlist::table).values(&insertable);
            query.execute(self.connection)?;
        }
        self.helper.after_entity_inserted(&entity)?;
        Ok(())
    }

    fn update_playlist(
        &self,
        entity: &Entity,
        body_data: EntityBodyData,
    ) -> RepoResult<EntityRevisionUpdateResult> {
        let prev_rev = entity.hdr.rev;
        let next_rev = prev_rev.next();
        {
            let (data_fmt, data_ver, data_blob) = body_data;
            let updatable = UpdatableEntity::bind(&next_rev, data_fmt, data_ver, &data_blob);
            let target = tbl_playlist::table.filter(
                tbl_playlist::uid
                    .eq(entity.hdr.uid.as_ref())
                    .and(tbl_playlist::rev_no.eq(prev_rev.no as i64))
                    .and(tbl_playlist::rev_ts.eq((prev_rev.ts.0).0)),
            );
            let repo_id = self
                .helper
                .before_entity_updated_or_removed(&entity.hdr.uid)?;
            let query = diesel::update(target).set(&updatable);
            let rows_affected: usize = query.execute(self.connection)?;
            debug_assert!(rows_affected <= 1);
            if rows_affected < 1 {
                let row = tbl_playlist::table
                    .select((tbl_playlist::rev_no, tbl_playlist::rev_ts))
                    .filter(tbl_playlist::uid.eq(entity.hdr.uid.as_ref()))
                    .first::<(i64, TickType)>(self.connection)
                    .optional()?;
                if let Some(row) = row {
                    let rev = EntityRevision {
                        no: row.0 as EntityVersionNumber,
                        ts: TickInstant(Ticks(row.1)),
                    };
                    return Ok(EntityRevisionUpdateResult::Current(rev));
                } else {
                    return Ok(EntityRevisionUpdateResult::NotFound);
                }
            }
            self.helper.after_entity_updated(repo_id, &entity.body)?;
        }
        Ok(EntityRevisionUpdateResult::Updated(prev_rev, next_rev))
    }

    fn delete_playlist(&self, uid: &EntityUid) -> RepoResult<Option<()>> {
        let target = tbl_playlist::table.filter(tbl_playlist::uid.eq(uid.as_ref()));
        let query = diesel::delete(target);
        self.helper.before_entity_updated_or_removed(uid)?;
        let rows_affected: usize = query.execute(self.connection)?;
        debug_assert!(rows_affected <= 1);
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            Ok(None)
        } else {
            Ok(Some(()))
        }
    }

    fn load_playlist(&self, uid: &EntityUid) -> RepoResult<Option<EntityData>> {
        tbl_playlist::table
            .filter(tbl_playlist::uid.eq(uid.as_ref()))
            .first::<QueryableEntityData>(self.connection)
            .optional()
            .map(|o| o.map(Into::into))
            .map_err(Into::into)
    }

    fn load_playlist_rev(&self, hdr: &EntityHeader) -> RepoResult<Option<EntityData>> {
        tbl_playlist::table
            .filter(
                tbl_playlist::uid
                    .eq(hdr.uid.as_ref())
                    .and(tbl_playlist::rev_no.eq(hdr.rev.no as i64))
                    .and(tbl_playlist::rev_ts.eq((hdr.rev.ts.0).0)),
            )
            .first::<QueryableEntityData>(self.connection)
            .optional()
            .map(|o| o.map(Into::into))
            .map_err(Into::into)
    }

    fn list_playlists(
        &self,
        r#type: Option<&str>,
        pagination: Pagination,
    ) -> RepoResult<Vec<EntityData>> {
        let mut target = tbl_playlist::table
            .then_order_by(tbl_playlist::rev_ts.desc())
            .into_boxed();

        // Filter by type
        if let Some(r#type) = r#type {
            target = target.filter(
                tbl_playlist::id.eq_any(
                    aux_playlist_brief::table
                        .select(aux_playlist_brief::playlist_id)
                        .filter(aux_playlist_brief::playlist_type.eq(r#type)),
                ),
            )
        }

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .load::<QueryableEntityData>(self.connection)
            .map(|v| v.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    fn list_playlist_briefs(
        &self,
        r#type: Option<&str>,
        pagination: Pagination,
    ) -> RepoResult<Vec<(EntityHeader, PlaylistBrief)>> {
        let mut target = tbl_playlist::table
            .inner_join(aux_playlist_brief::table)
            .then_order_by(tbl_playlist::rev_ts.desc())
            .into_boxed();

        // Filter by type
        if let Some(r#type) = r#type {
            target = target.filter(aux_playlist_brief::playlist_type.eq(r#type));
        }

        // Pagination
        target = apply_pagination(target, pagination);

        target
            .select((
                tbl_playlist::id,
                tbl_playlist::uid,
                tbl_playlist::rev_no,
                tbl_playlist::rev_ts,
                aux_playlist_brief::name,
                aux_playlist_brief::desc,
                aux_playlist_brief::playlist_type,
                aux_playlist_brief::color_rgb,
                aux_playlist_brief::color_idx,
                aux_playlist_brief::geoloc_lat,
                aux_playlist_brief::geoloc_lon,
                aux_playlist_brief::tracks_count,
                aux_playlist_brief::entries_count,
                aux_playlist_brief::entries_added_min,
                aux_playlist_brief::entries_added_max,
            ))
            .load::<QueryableBrief>(self.connection)
            .map(|v| {
                v.into_iter()
                    .map(Into::into)
                    .map(|(_, hdr, brief)| (hdr, brief))
                    .collect()
            })
            .map_err(Into::into)
    }

    fn count_playlist_entries(&self, uid: &EntityUid) -> RepoResult<Option<usize>> {
        Ok(aux_playlist_track::table
            .filter(
                aux_playlist_track::playlist_id.eq_any(
                    tbl_playlist::table
                        .filter(tbl_playlist::uid.eq(uid.as_ref()))
                        .select(tbl_playlist::id),
                ),
            )
            .select(dsl::sum(aux_playlist_track::track_ref_count))
            .first::<Option<BigDecimal>>(self.connection)
            .optional()
            .map(|opt_sum| {
                opt_sum.flatten().map(|sum| {
                    // TODO: Re-enable assertion for bigdecimal >= 0.1.0
                    //debug_assert!(sum.is_integer());
                    debug_assert!(sum.sign() != Sign::Minus);
                    sum.to_bigint()
                        .expect("uncountable playlist entries")
                        .to_usize()
                        .expect("too many playlist entries")
                })
            })?)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
