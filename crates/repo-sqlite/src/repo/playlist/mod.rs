// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Range;

use crate::{
    db::{
        playlist::{models::*, schema::*},
        playlist_entry as playlist_entry_db,
        track::schema as track_schema,
    },
    prelude::*,
};

use aoide_core::{
    entity::EntityRevision,
    playlist::{track::Item as TrackItem, *},
    util::clock::*,
};

use aoide_core_api::playlist::EntityWithEntriesSummary;
use aoide_repo::{collection::RecordId as CollectionId, playlist::*, track::EntityRepo as _};
use diesel::dsl::count_star;

impl<'db> EntityRepo for crate::Connection<'db> {
    fn resolve_playlist_entity_revision(
        &self,
        uid: &EntityUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        playlist::table
            .select((
                playlist::row_id,
                playlist::row_created_ms,
                playlist::row_updated_ms,
                playlist::entity_rev,
            ))
            .filter(playlist::entity_uid.eq(uid.as_ref()))
            .first::<(RowId, TimestampMillis, TimestampMillis, i64)>(self.as_ref())
            .map_err(repo_error)
            .map(|(row_id, row_created_ms, row_updated_ms, entity_rev)| {
                let header = RecordHeader {
                    id: row_id.into(),
                    created_at: DateTime::new_timestamp_millis(row_created_ms),
                    updated_at: DateTime::new_timestamp_millis(row_updated_ms),
                };
                (header, entity_revision_from_sql(entity_rev))
            })
    }

    fn touch_playlist_entity_revision(
        &self,
        entity_header: &EntityHeader,
        updated_at: DateTime,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        let EntityHeader { uid, rev } = entity_header;
        let next_rev = rev
            .next()
            .ok_or_else(|| anyhow::anyhow!("no next revision"))?;
        let touchable = TouchableRecord::bind(updated_at, next_rev);
        let target = playlist::table
            .filter(playlist::entity_uid.eq(uid.as_ref()))
            .filter(playlist::entity_rev.eq(entity_revision_to_sql(*rev)));
        let query = diesel::update(target).set(&touchable);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        let resolved = self.resolve_playlist_entity_revision(uid)?;
        if rows_affected < 1 {
            // Successfully resolved by UID, but not touched due to revision conflict
            return Err(RepoError::Conflict);
        }
        Ok(resolved)
    }

    fn update_playlist_entity(
        &self,
        id: RecordId,
        updated_at: DateTime,
        updated_entity: &Entity,
    ) -> RepoResult<()> {
        let updatable =
            UpdatableRecord::bind(updated_at, updated_entity.hdr.rev, &updated_entity.body);
        let target = playlist::table.filter(playlist::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&updatable);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_playlist_entity(&self, id: RecordId) -> RepoResult<(RecordHeader, Entity)> {
        let record = playlist::table
            .filter(playlist::row_id.eq(RowId::from(id)))
            .first::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;
        let (record_header, _, entity) = record.into();
        Ok((record_header, entity))
    }

    fn load_playlist_entity_with_entries(&self, id: RecordId) -> RepoResult<EntityWithEntries> {
        let (_, entity) = self.load_playlist_entity(id)?;
        let entries = self.load_all_playlist_entries(id)?;
        Ok((entity, entries).into())
    }

    fn purge_playlist_entity(&self, id: RecordId) -> RepoResult<()> {
        let target = playlist::table.filter(playlist::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}

impl<'db> CollectionRepo for crate::Connection<'db> {
    fn insert_playlist_entity(
        &self,
        collection_id: CollectionId,
        created_at: DateTime,
        created_entity: &Entity,
    ) -> RepoResult<RecordId> {
        let insertable = InsertableRecord::bind(collection_id, created_at, created_entity);
        let query = diesel::insert_into(playlist::table).values(&insertable);
        let rows_affected = query.execute(self.as_ref()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        self.resolve_playlist_id(&created_entity.hdr.uid)
    }

    fn load_playlist_entities_with_entries_summary(
        &self,
        collection_id: CollectionId,
        kind: Option<&str>,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = EntityWithEntriesSummary,
        >,
    ) -> RepoResult<()> {
        let mut target = playlist::table
            .filter(playlist::collection_id.eq(RowId::from(collection_id)))
            .order_by(playlist::row_updated_ms.desc())
            .into_boxed();

        // Kind
        if let Some(kind) = kind {
            target = target.filter(playlist::kind.eq(kind));
        }

        // Pagination
        if let Some(pagination) = pagination {
            target = apply_pagination(target, pagination);
        }

        let records = target
            .load::<QueryableRecord>(self.as_ref())
            .map_err(repo_error)?;

        collector.reserve(records.len());
        for record in records {
            let (record_header, record_collection_id, entity) = record.into();
            debug_assert_eq!(collection_id, record_collection_id);
            let entries = self.load_playlist_entries_summary(record_header.id)?;
            collector.collect(record_header, EntityWithEntriesSummary { entity, entries });
        }
        Ok(())
    }
}

fn min_playlist_entry_ordering(
    db: &crate::Connection<'_>,
    playlist_id: RecordId,
) -> RepoResult<Option<i64>> {
    use playlist_entry_db::schema::*;
    playlist_entry::table
        .select(diesel::dsl::min(playlist_entry::ordering))
        .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
        .first::<Option<i64>>(db.as_ref())
        .map_err(repo_error)
}

fn max_playlist_entry_ordering(
    db: &crate::Connection<'_>,
    playlist_id: RecordId,
) -> RepoResult<Option<i64>> {
    use playlist_entry_db::schema::*;
    playlist_entry::table
        .select(diesel::dsl::max(playlist_entry::ordering))
        .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
        .first::<Option<i64>>(db.as_ref())
        .map_err(repo_error)
}

fn shift_playlist_entries_forward(
    db: &crate::Connection<'_>,
    playlist_id: RecordId,
    old_min_ordering: i64,
    delta_ordering: i64,
) -> RepoResult<usize> {
    use playlist_entry_db::schema::*;
    debug_assert!(delta_ordering > 0);
    // Unfortunately, the ordering column cannot be incremented by
    // a single SQL statement. The update fails with a UNIQUE constraint
    // violation if the entries are not updated in descending order
    // of the ordering column to ensure uniqueness at any time.
    let row_ids = playlist_entry::table
        .select(playlist_entry::row_id)
        .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
        .filter(playlist_entry::ordering.ge(old_min_ordering))
        .order_by(playlist_entry::ordering.desc())
        .load::<RowId>(db.as_ref())
        .map_err(repo_error)?;
    let mut rows_updated = 0;
    for row_id in row_ids {
        rows_updated +=
            diesel::update(playlist_entry::table.filter(playlist_entry::row_id.eq(row_id)))
                .set(
                    playlist_entry::ordering
                        .eq(diesel::dsl::sql(&format!("ordering+{delta_ordering}"))),
                )
                .execute(db.as_ref())
                .map_err(repo_error)?;
    }
    Ok(rows_updated)
}

fn reverse_all_playlist_entries_tail(
    db: &crate::Connection<'_>,
    playlist_id: RecordId,
    old_min_ordering: i64,
    new_max_ordering: i64,
) -> RepoResult<usize> {
    use playlist_entry_db::schema::*;
    // Unfortunately, the ordering column cannot be incremented by
    // a single SQL statement. The update fails with a UNIQUE constraint
    // violation if the entries are not updated in descending order
    // of the ordering column to ensure uniqueness at any time.
    let row_ids = playlist_entry::table
        .select(playlist_entry::row_id)
        .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
        .filter(playlist_entry::ordering.ge(old_min_ordering))
        .order_by(playlist_entry::ordering)
        .load::<RowId>(db.as_ref())
        .map_err(repo_error)?;
    let mut rows_updated = 0;
    let mut ordering = new_max_ordering;
    for row_id in row_ids {
        rows_updated +=
            diesel::update(playlist_entry::table.filter(playlist_entry::row_id.eq(row_id)))
                .set(playlist_entry::ordering.eq(ordering))
                .execute(db.as_ref())
                .map_err(repo_error)?;
        ordering = ordering.saturating_sub(-1);
    }
    Ok(rows_updated)
}

fn load_playlist_entry_records(
    db: &crate::Connection<'_>,
    playlist_id: RecordId,
) -> RepoResult<Vec<playlist_entry_db::models::QueryableRecord>> {
    use playlist_entry_db::{models::*, schema::*};
    use track_schema::*;
    playlist_entry::table
        .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
        .left_outer_join(track_schema::track::table)
        .select((
            playlist_entry::playlist_id,
            playlist_entry::ordering,
            playlist_entry::track_id,
            track::entity_uid.nullable(),
            playlist_entry::added_at,
            playlist_entry::added_ms,
            playlist_entry::title,
            playlist_entry::notes,
        ))
        .order_by(playlist_entry::ordering)
        .load::<QueryableRecord>(db.as_ref())
        .map_err(repo_error)
}

// TODO: Overwrite remaining default implementations of EntryRepo that are inefficient,
// e.g. for moving and shuffling playlist entries.
impl<'db> EntryRepo for crate::Connection<'db> {
    fn load_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<Vec<Entry>> {
        let records = load_playlist_entry_records(self, playlist_id)?;
        let mut entries = Vec::with_capacity(records.len());
        for record in records {
            let (record_playlist_id, _ordering, _track_id, entry) = record.into();
            debug_assert_eq!(playlist_id, record_playlist_id);
            entries.push(entry);
        }
        Ok(entries)
    }

    fn count_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
            .select(count_star())
            .first::<i64>(self.as_ref())
            .map(|count| count as usize)
            .map_err(repo_error)
    }

    fn load_playlist_entries_summary(&self, playlist_id: RecordId) -> RepoResult<EntriesSummary> {
        use playlist_entry_db::schema::*;
        let entries_count = self.count_playlist_entries(playlist_id)?;
        let tracks_count = playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
            .select(count_star())
            .filter(playlist_entry::track_id.is_not_null())
            .first::<i64>(self.as_ref())
            .map(|count| count as usize)
            .map_err(repo_error)?;
        debug_assert!(tracks_count <= entries_count);
        let added_at_minmax = if entries_count > 0 {
            let added_at_min = playlist_entry::table
                .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
                .select((playlist_entry::added_at, playlist_entry::added_ms))
                .order_by(playlist_entry::added_ms.asc())
                .first::<(String, TimestampMillis)>(self.as_ref())
                .optional()
                .map(|opt| opt.map(|(at, ms)| parse_datetime(&at, ms)))
                .map_err(repo_error)?;
            let added_at_max = playlist_entry::table
                .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
                .select((playlist_entry::added_at, playlist_entry::added_ms))
                .order_by(playlist_entry::added_ms.desc())
                .first::<(String, TimestampMillis)>(self.as_ref())
                .optional()
                .map(|opt| opt.map(|(at, ms)| parse_datetime(&at, ms)))
                .map_err(repo_error)?;
            debug_assert_eq!(added_at_min.is_some(), added_at_max.is_some());
            if let (Some(added_at_min), Some(added_at_max)) = (added_at_min, added_at_max) {
                Some((added_at_min, added_at_max))
            } else {
                None
            }
        } else {
            None
        };
        Ok(EntriesSummary {
            total_count: entries_count,
            added_at_minmax,
            tracks: TracksSummary {
                total_count: tracks_count,
            },
        })
    }

    fn append_playlist_entries(
        &self,
        playlist_id: RecordId,
        new_entries: &[Entry],
    ) -> RepoResult<()> {
        if new_entries.is_empty() {
            return Ok(());
        }
        use playlist_entry_db::{models::*, schema::*};
        let max_ordering = max_playlist_entry_ordering(self, playlist_id)?.unwrap_or(-1);
        let mut ordering = max_ordering;
        let created_at = DateTime::now_utc();
        for entry in new_entries {
            ordering = ordering.saturating_add(1);
            let track_id = match &entry.item {
                Item::Separator => None,
                Item::Track(TrackItem { uid }) => Some(self.resolve_track_id(uid)?),
            };
            let insertable =
                InsertableRecord::bind(playlist_id, track_id, ordering, created_at, entry);
            let rows_affected = diesel::insert_into(playlist_entry::table)
                .values(&insertable)
                .execute(self.as_ref())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
        }
        Ok(())
    }

    fn prepend_playlist_entries(
        &self,
        playlist_id: RecordId,
        new_entries: &[Entry],
    ) -> RepoResult<()> {
        if new_entries.is_empty() {
            return Ok(());
        }
        use playlist_entry_db::{models::*, schema::*};
        let min_ordering = min_playlist_entry_ordering(self, playlist_id)?.unwrap_or(0);
        // TODO: Ordering range checks and adjustments when needed!
        debug_assert!(new_entries.len() as i64 >= 0);
        let mut ordering = min_ordering.saturating_sub(new_entries.len() as i64);
        let created_at = DateTime::now_utc();
        for entry in new_entries {
            let track_id = match &entry.item {
                Item::Separator => None,
                Item::Track(TrackItem { uid }) => Some(self.resolve_track_id(uid)?),
            };
            let insertable =
                InsertableRecord::bind(playlist_id, track_id, ordering, created_at, entry);
            let rows_affected = diesel::insert_into(playlist_entry::table)
                .values(&insertable)
                .execute(self.as_ref())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
            ordering = ordering.saturating_add(1);
        }
        Ok(())
    }

    fn remove_playlist_entries(
        &self,
        playlist_id: RecordId,
        index_range: &Range<usize>,
    ) -> RepoResult<usize> {
        if index_range.is_empty() {
            return Ok(0);
        }
        use playlist_entry_db::schema::*;
        let offset = index_range.start as i64;
        debug_assert!(offset >= 0);
        let limit = index_range.len() as i64;
        debug_assert!(limit >= 0);
        let delete_row_ids_subselect = playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
            .select(playlist_entry::row_id)
            .order_by(playlist_entry::ordering)
            .offset(offset)
            .limit(limit);
        // TODO: Using the subelect in the delete statement without temporarily
        // loading the corresponding row ids didn't work.
        // let delete_target =
        //    playlist_entry::table.filter(playlist_entry::row_id.eq_any(delete_row_ids_subselect));
        let delete_target = playlist_entry::table.filter(
            playlist_entry::row_id.eq_any(
                delete_row_ids_subselect
                    .load::<i64>(self.as_ref())
                    .map_err(repo_error)?,
            ),
        );
        let rows_deleted: usize = diesel::delete(delete_target)
            .execute(self.as_ref())
            .map_err(repo_error)?;
        debug_assert!(rows_deleted <= index_range.len());
        Ok(rows_deleted)
    }

    fn remove_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        let rows_deleted: usize = diesel::delete(
            playlist_entry::table.filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id))),
        )
        .execute(self.as_ref())
        .map_err(repo_error)?;
        Ok(rows_deleted)
    }

    fn reverse_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        let min_ordering = min_playlist_entry_ordering(self, playlist_id)?;
        let max_ordering = max_playlist_entry_ordering(self, playlist_id)?;
        let rows_updated =
            if let (Some(min_ordering), Some(max_ordering)) = (min_ordering, max_ordering) {
                let rows_updated;
                if (min_ordering.is_negative() && max_ordering.is_positive())
                    || (min_ordering.is_positive() && max_ordering.is_negative())
                {
                    // Shift forward and reverse
                    let new_max_ordering = max_ordering
                        .saturating_add(1)
                        .max(self.count_playlist_entries(playlist_id)? as i64);
                    debug_assert!(new_max_ordering > max_ordering);
                    rows_updated = reverse_all_playlist_entries_tail(
                        self,
                        playlist_id,
                        min_ordering,
                        new_max_ordering,
                    )?;
                    debug_assert_eq!(rows_updated, self.count_playlist_entries(playlist_id)?);
                } else {
                    // Optimization: Negate ordering
                    let target = playlist_entry::table
                        .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)));
                    rows_updated = diesel::update(target)
                        .set(playlist_entry::ordering.eq(diesel::dsl::sql("-ordering")))
                        .execute(self.as_ref())
                        .map_err(repo_error)?;
                }
                rows_updated
            } else {
                debug_assert!(min_ordering.is_none());
                debug_assert!(max_ordering.is_none());
                debug_assert!(self.count_playlist_entries(playlist_id)? == 0);
                0
            };
        Ok(rows_updated)
    }

    fn insert_playlist_entries(
        &self,
        playlist_id: RecordId,
        before_index: usize,
        new_entries: &[Entry],
    ) -> RepoResult<()> {
        if new_entries.is_empty() {
            return Ok(());
        }
        use playlist_entry_db::{models::*, schema::*};
        let offset = before_index as i64;
        debug_assert!(offset >= 0);
        // The newly inserted entries will be assigned ordering numbers
        // from prev_ordering + 1 to prev_ordering + new_entries.len()
        let prev_ordering = if offset > 0 {
            playlist_entry::table
                .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
                .select(playlist_entry::ordering)
                .order_by(playlist_entry::ordering)
                .offset(offset - 1)
                .first::<i64>(self.as_ref())
                .optional()
                .map_err(repo_error)?
        } else {
            None
        };
        // Reordering is only needed if one or more entries follow the deleted range
        let next_ordering = playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(playlist_id)))
            .select(playlist_entry::ordering)
            .order_by(playlist_entry::ordering)
            .offset(offset)
            .first::<i64>(self.as_ref())
            .optional()
            .map_err(repo_error)?;
        debug_assert!(new_entries.len() as i64 >= 0);
        let new_ordering_range = if let Some(next_ordering) = next_ordering {
            // TODO: Ordering range checks and adjustments when needed!
            let prev_ordering =
                prev_ordering.unwrap_or((next_ordering - 1) - new_entries.len() as i64);
            let new_ordering_range =
                (prev_ordering + 1)..(prev_ordering + 1 + new_entries.len() as i64);
            debug_assert!(new_ordering_range.start <= new_ordering_range.end);
            if next_ordering < new_ordering_range.end {
                // Shift subsequent entries
                let delta_ordering = new_ordering_range.end - next_ordering;
                let rows_updated = shift_playlist_entries_forward(
                    self,
                    playlist_id,
                    next_ordering,
                    delta_ordering,
                )?;
                log::debug!(
                    "Reordered {rows_updated} entries of playlist {row_id} before inserting {num_new_entries} entries",
                    row_id = RowId::from(playlist_id),
                    num_new_entries = new_entries.len(),
                );
            }
            new_ordering_range
        } else {
            // TODO: Ordering range checks and adjustments when needed!
            let prev_ordering = prev_ordering.unwrap_or(-1);
            let new_ordering_range =
                (prev_ordering + 1)..((prev_ordering + 1) + new_entries.len() as i64);
            debug_assert!(new_ordering_range.start <= new_ordering_range.end);
            new_ordering_range
        };
        let mut ordering = new_ordering_range.start;
        let created_at = DateTime::now_utc();
        for entry in new_entries {
            let track_id = match &entry.item {
                Item::Separator => None,
                Item::Track(TrackItem { uid }) => Some(self.resolve_track_id(uid)?),
            };
            let insertable =
                InsertableRecord::bind(playlist_id, track_id, ordering, created_at, entry);
            let rows_affected = diesel::insert_into(playlist_entry::table)
                .values(&insertable)
                .execute(self.as_ref())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
            ordering = ordering.saturating_add(1);
        }
        Ok(())
    }

    fn copy_all_playlist_entries(
        &self,
        source_playlist_id: RecordId,
        target_playlist_id: RecordId,
    ) -> RepoResult<usize> {
        use playlist_entry_db::{models::*, schema::*};
        let records = load_playlist_entry_records(self, source_playlist_id)?;
        let copied_count = records.len();
        let created_at = DateTime::now_utc();
        for record in records {
            let (_playlist_id, ordering, track_id, entry) = record.into();
            let insertable =
                InsertableRecord::bind(target_playlist_id, track_id, ordering, created_at, &entry);
            let rows_affected = diesel::insert_into(playlist_entry::table)
                .values(&insertable)
                .execute(self.as_ref())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
        }
        Ok(copied_count)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
