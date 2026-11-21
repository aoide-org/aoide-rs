// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{collections::BTreeMap, ops::Range};

use anyhow::anyhow;
use diesel::{
    dsl::{count, count_star, max, min},
    prelude::*,
};

use aoide_core::{
    EntityRevision, PlaylistEntity, PlaylistUid,
    playlist::{
        EntityHeader, EntityWithEntries, EntriesSummary, Entry, Item, TrackItem, TracksSummary,
    },
    util::clock::*,
};
use aoide_core_api::{Pagination, playlist::EntityWithEntriesSummary};
use aoide_repo::{
    CollectionId, RepoError, RepoResult, ReservableRecordCollector, TrackId, playlist::*,
    track::EntityRepo as _,
};

use crate::{
    Connection, RowId,
    db::{
        playlist::{models::*, schema::*},
        playlist_entry as playlist_entry_db,
        track::schema as track_schema,
    },
    repo_error,
    util::{
        apply_pagination,
        clock::timestamp_millis,
        entity::{decode_entity_revision, encode_entity_revision, encode_entity_uid},
    },
};

impl EntityRepo for Connection<'_> {
    fn resolve_playlist_entity_revision(
        &mut self,
        uid: &PlaylistUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        playlist::table
            .select((
                playlist::row_id,
                playlist::row_created_ms,
                playlist::row_updated_ms,
                playlist::entity_rev,
            ))
            .filter(playlist::entity_uid.eq(encode_entity_uid(uid).as_str()))
            .get_result::<(RowId, TimestampMillis, TimestampMillis, i64)>(self.as_mut())
            .map_err(repo_error)
            .map(|(row_id, row_created_ms, row_updated_ms, entity_rev)| {
                let header = RecordHeader {
                    id: row_id.into(),
                    created_at: UtcDateTimeMs::from_unix_timestamp_millis(row_created_ms),
                    updated_at: UtcDateTimeMs::from_unix_timestamp_millis(row_updated_ms),
                };
                (header, decode_entity_revision(entity_rev))
            })
    }

    fn touch_playlist_entity_revision(
        &mut self,
        entity_header: &EntityHeader,
        updated_at: UtcDateTimeMs,
    ) -> RepoResult<(RecordHeader, EntityRevision)> {
        let EntityHeader { uid, rev } = entity_header;
        let next_rev = rev
            .next()
            .ok_or_else(|| RepoError::Other(anyhow!("no next revision")))?;
        let touchable = TouchableRecord::bind(updated_at, next_rev);
        let encoded_uid = encode_entity_uid(uid);
        let target = playlist::table
            .filter(playlist::entity_uid.eq(encoded_uid.as_str()))
            .filter(playlist::entity_rev.eq(encode_entity_revision(*rev)));
        let query = diesel::update(target).set(&touchable);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        let resolved = self.resolve_playlist_entity_revision(uid)?;
        if rows_affected < 1 {
            // Successfully resolved by UID, but not touched due to revision conflict
            return Err(RepoError::Conflict);
        }
        Ok(resolved)
    }

    fn update_playlist_entity(
        &mut self,
        id: RecordId,
        updated_at: UtcDateTimeMs,
        updated_entity: &PlaylistEntity,
    ) -> RepoResult<()> {
        let updatable =
            UpdatableRecord::bind(updated_at, updated_entity.hdr.rev, &updated_entity.body);
        let target = playlist::table.filter(playlist::row_id.eq(RowId::from(id)));
        let query = diesel::update(target).set(&updatable);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn load_playlist_entity(&mut self, id: RecordId) -> RepoResult<(RecordHeader, PlaylistEntity)> {
        let record = playlist::table
            .filter(playlist::row_id.eq(RowId::from(id)))
            .get_result::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)?;
        let (record_header, _, entity) = record.into();
        Ok((record_header, entity))
    }

    fn load_playlist_entity_with_entries(
        &mut self,
        id: RecordId,
    ) -> RepoResult<(RecordHeader, EntityWithEntries)> {
        let (record_header, entity) = self.load_playlist_entity(id)?;
        let entries = self.load_all_playlist_entries(id)?;
        let entity_with_entries = EntityWithEntries::from((entity, entries));
        Ok((record_header, entity_with_entries))
    }

    fn purge_playlist_entity(&mut self, id: RecordId) -> RepoResult<()> {
        let target = playlist::table.filter(playlist::row_id.eq(RowId::from(id)));
        let query = diesel::delete(target);
        let rows_affected: usize = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert!(rows_affected <= 1);
        if rows_affected < 1 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    fn insert_playlist_entity(
        &mut self,
        collection_id: Option<CollectionId>,
        created_at: UtcDateTimeMs,
        created_entity: &PlaylistEntity,
    ) -> RepoResult<RecordId> {
        let insertable = InsertableRecord::bind(collection_id, created_at, created_entity);
        let query = insertable.insert_into(playlist::table);
        let rows_affected = query.execute(self.as_mut()).map_err(repo_error)?;
        debug_assert_eq!(1, rows_affected);
        self.resolve_playlist_id(&created_entity.hdr.uid)
    }

    fn load_playlist_entities_with_entries_summary(
        &mut self,
        collection_filter: Option<CollectionFilter>,
        kind_filter: Option<KindFilter<'_>>,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = EntityWithEntriesSummary>,
    ) -> RepoResult<()> {
        let mut target = playlist::table
            .order_by(playlist::row_updated_ms.desc())
            .into_boxed();

        if let Some(collection_filter) = collection_filter {
            let CollectionFilter { id: collection_id } = collection_filter;
            if let Some(collection_id) = collection_id {
                target =
                    target.filter(playlist::collection_id.eq(Some(RowId::from(collection_id))));
            } else {
                // Note: playlist::collection_id.eq(None) does not match NULL!
                // <https://github.com/diesel-rs/diesel/issues/1306>
                target = target.filter(playlist::collection_id.is_null());
            }
        }

        if let Some(kind_filter) = kind_filter {
            match kind_filter {
                KindFilter::IsNone => {
                    // Note: playlist::kind.eq(None) does not match NULL!
                    // <https://github.com/diesel-rs/diesel/issues/1306>
                    target = target.filter(playlist::kind.is_null());
                }
                KindFilter::Equal(kind) => {
                    target = target.filter(playlist::kind.eq(kind));
                }
                KindFilter::NotEqual(kind) => {
                    target = target.filter(playlist::kind.ne(kind));
                }
            }
        }

        if let Some(pagination) = pagination {
            target = apply_pagination(target, pagination);
        }

        let records = target
            .load::<QueryableRecord>(self.as_mut())
            .map_err(repo_error)?;

        collector.reserve(records.len());
        let mut records = records.into_iter().peekable();
        while records.peek().is_some() {
            let chunk = records
                .by_ref()
                .take(LOAD_ENTRIES_SUMMARIES_CHUNK_SIZE)
                .collect::<Vec<_>>();
            let ids = chunk
                .iter()
                .map(|record| RecordId::from(record.row_id))
                .collect::<Vec<_>>();
            let mut entries_summaries = load_entries_summaries(self, &ids)?;
            for record in chunk {
                let (record_header, _collection_id, entity) = record.into();
                let entries = entries_summaries
                    .remove(&record_header.id)
                    .unwrap_or(EntriesSummary::EMPTY);
                collector.collect(record_header, EntityWithEntriesSummary { entity, entries });
            }
        }
        Ok(())
    }
}

/// Maximum number of playlists whose entries are summarized by a single query.
///
/// Limits the number of bound SQL parameters.
const LOAD_ENTRIES_SUMMARIES_CHUNK_SIZE: usize = 500;

/// Summarize the entries of multiple playlists with a single query.
///
/// Playlists without entries are omitted from the result.
fn load_entries_summaries(
    db: &mut crate::Connection<'_>,
    ids: &[RecordId],
) -> RepoResult<BTreeMap<RecordId, EntriesSummary>> {
    use playlist_entry_db::schema::*;
    let rows = playlist_entry::table
        .filter(playlist_entry::playlist_id.eq_any(ids.iter().copied().map(RowId::from)))
        .group_by(playlist_entry::playlist_id)
        .select((
            playlist_entry::playlist_id,
            count_star(),
            // Only counts entries with a track, i.e. ignores NULL values
            count(playlist_entry::track_id),
            count(playlist_entry::track_id).aggregate_distinct(),
            min(playlist_entry::added_ms),
            max(playlist_entry::added_ms),
        ))
        .load::<(
            RowId,
            i64,
            i64,
            i64,
            Option<TimestampMillis>,
            Option<TimestampMillis>,
        )>(db.as_mut())
        .map_err(repo_error)?;
    let summaries = rows
        .into_iter()
        .map(
            |(
                playlist_id,
                entries_count,
                tracks_count,
                distinct_tracks_count,
                added_ms_min,
                added_ms_max,
            )| {
                debug_assert!(tracks_count <= entries_count);
                debug_assert!(distinct_tracks_count <= tracks_count);
                let added_ts_minmax = added_ms_min
                    .zip(added_ms_max)
                    .map(|(min, max)| (timestamp_millis(min), timestamp_millis(max)));
                let summary = EntriesSummary {
                    total_count: entries_count as usize,
                    added_ts_minmax,
                    tracks: TracksSummary {
                        total_count: tracks_count as usize,
                        distinct_count: distinct_tracks_count as usize,
                    },
                };
                (playlist_id.into(), summary)
            },
        )
        .collect();
    Ok(summaries)
}

fn min_playlist_entry_ordering(
    db: &mut crate::Connection<'_>,
    id: RecordId,
) -> RepoResult<Option<i64>> {
    use playlist_entry_db::schema::*;
    playlist_entry::table
        .select(diesel::dsl::min(playlist_entry::ordering))
        .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
        .get_result::<Option<i64>>(db.as_mut())
        .map_err(repo_error)
}

fn max_playlist_entry_ordering(
    db: &mut crate::Connection<'_>,
    id: RecordId,
) -> RepoResult<Option<i64>> {
    use playlist_entry_db::schema::*;
    playlist_entry::table
        .select(diesel::dsl::max(playlist_entry::ordering))
        .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
        .get_result::<Option<i64>>(db.as_mut())
        .map_err(repo_error)
}

// The ordering of entries cannot be modified by a single SQL statement,
// because the UNIQUE constraint on (playlist_id, ordering) is checked
// for each row during the update and not only at the end. Instead the
// entries are moved into a free range below the current minimum ordering
// first and then into their final position. All intermediate and final
// orderings are disjoint from all current orderings in both steps.

/// Increment the ordering of all entries starting at `old_min_ordering`
/// by `delta_ordering`.
fn shift_playlist_entries_forward(
    db: &mut crate::Connection<'_>,
    id: RecordId,
    old_min_ordering: i64,
    delta_ordering: i64,
) -> RepoResult<usize> {
    use playlist_entry_db::schema::*;
    debug_assert!(delta_ordering > 0);
    let Some(min_ordering) = min_playlist_entry_ordering(db, id)? else {
        return Ok(0);
    };
    debug_assert!(min_ordering <= old_min_ordering);
    let target = playlist_entry::table.filter(playlist_entry::playlist_id.eq(RowId::from(id)));
    // ordering -> (min_ordering - 1) - (ordering - old_min_ordering) < min_ordering
    let rows_updated = diesel::update(target.filter(playlist_entry::ordering.ge(old_min_ordering)))
        .set(
            playlist_entry::ordering
                .eq(playlist_entry::ordering * -1 + (min_ordering - 1 + old_min_ordering)),
        )
        .execute(db.as_mut())
        .map_err(repo_error)?;
    // Invert the first step and add delta_ordering
    diesel::update(target.filter(playlist_entry::ordering.lt(min_ordering)))
        .set(
            playlist_entry::ordering.eq(playlist_entry::ordering * -1
                + (min_ordering - 1 + old_min_ordering + delta_ordering)),
        )
        .execute(db.as_mut())
        .map_err(repo_error)?;
    Ok(rows_updated)
}

/// Reverse the ordering of all entries within the same range of orderings.
fn reverse_playlist_entries(
    db: &mut crate::Connection<'_>,
    id: RecordId,
    min_ordering: i64,
    max_ordering: i64,
) -> RepoResult<usize> {
    use playlist_entry_db::schema::*;
    let target = playlist_entry::table.filter(playlist_entry::playlist_id.eq(RowId::from(id)));
    // ordering -> (min_ordering - 1) - (ordering - min_ordering) < min_ordering
    let rows_updated = diesel::update(target)
        .set(playlist_entry::ordering.eq(playlist_entry::ordering * -1 + (2 * min_ordering - 1)))
        .execute(db.as_mut())
        .map_err(repo_error)?;
    // Move back into the original range of orderings
    diesel::update(target)
        .set(
            playlist_entry::ordering
                .eq(playlist_entry::ordering + (max_ordering - min_ordering + 1)),
        )
        .execute(db.as_mut())
        .map_err(repo_error)?;
    Ok(rows_updated)
}

fn load_playlist_entry_records(
    db: &mut crate::Connection<'_>,
    id: RecordId,
) -> RepoResult<Vec<playlist_entry_db::models::QueryableRecord>> {
    use playlist_entry_db::{models::*, schema::*};
    use track_schema::*;
    playlist_entry::table
        .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
        .left_outer_join(track_schema::track::table)
        .select((
            playlist_entry::playlist_id,
            playlist_entry::ordering,
            playlist_entry::track_id,
            track::entity_uid.nullable(),
            playlist_entry::added_ms,
            playlist_entry::title,
            playlist_entry::notes,
            playlist_entry::item_data,
        ))
        .order_by(playlist_entry::ordering)
        .load::<QueryableRecord>(db.as_mut())
        .map_err(repo_error)
}

// TODO: Overwrite remaining default implementations of EntryRepo that are inefficient,
// e.g. for moving and shuffling playlist entries.
impl EntryRepo for crate::Connection<'_> {
    fn load_all_playlist_entries(&mut self, id: RecordId) -> RepoResult<Vec<Entry>> {
        let records = load_playlist_entry_records(self, id)?;
        let mut entries = Vec::with_capacity(records.len());
        for record in records {
            let (record_id, _ordering, _track_id, entry) = record.into();
            debug_assert_eq!(id, record_id);
            entries.push(entry);
        }
        Ok(entries)
    }

    fn count_playlist_entries(&mut self, id: RecordId) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
            .count()
            .get_result::<i64>(self.as_mut())
            .map(|count| count as usize)
            .map_err(repo_error)
    }

    fn load_playlist_tracks_summary(&mut self, id: RecordId) -> RepoResult<TracksSummary> {
        use playlist_entry_db::schema::*;
        playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
            .select((
                count_star(),
                count(playlist_entry::track_id).aggregate_distinct(),
            ))
            .filter(playlist_entry::track_id.is_not_null())
            .get_result::<(i64, i64)>(self.as_mut())
            .map(|(total_count, distinct_count)| {
                debug_assert!(total_count >= 0);
                debug_assert!(distinct_count >= 0);
                debug_assert!(distinct_count <= total_count);
                TracksSummary {
                    total_count: total_count as usize,
                    distinct_count: distinct_count as usize,
                }
            })
            .map_err(repo_error)
    }

    fn count_playlist_single_track_entries(
        &mut self,
        id: RecordId,
        track_id: TrackId,
    ) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
            .filter(playlist_entry::track_id.eq(RowId::from(track_id)))
            .count()
            .get_result::<i64>(self.as_mut())
            .map(|count| count as usize)
            .map_err(repo_error)
    }

    fn load_playlist_entries_summary(&mut self, id: RecordId) -> RepoResult<EntriesSummary> {
        Ok(load_entries_summaries(self, &[id])?
            .remove(&id)
            .unwrap_or(EntriesSummary::EMPTY))
    }

    fn append_playlist_entries(&mut self, id: RecordId, new_entries: &[Entry]) -> RepoResult<()> {
        use playlist_entry_db::{models::*, schema::*};
        if new_entries.is_empty() {
            return Ok(());
        }
        let max_ordering = max_playlist_entry_ordering(self, id)?.unwrap_or(-1);
        let mut ordering = max_ordering;
        let created_at = UtcDateTimeMs::now();
        for entry in new_entries {
            ordering = ordering.saturating_add(1);
            let track_id = match &entry.item {
                Item::Separator(_) => None,
                Item::Track(TrackItem { uid }) => Some(self.resolve_track_id(uid)?),
            };
            let insertable = InsertableRecord::bind(id, track_id, ordering, &created_at, entry);
            let rows_affected = insertable
                .insert_into(playlist_entry::table)
                .execute(self.as_mut())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
        }
        Ok(())
    }

    fn prepend_playlist_entries(&mut self, id: RecordId, new_entries: &[Entry]) -> RepoResult<()> {
        use playlist_entry_db::{models::*, schema::*};
        if new_entries.is_empty() {
            return Ok(());
        }
        let min_ordering = min_playlist_entry_ordering(self, id)?.unwrap_or(0);
        // TODO: Ordering range checks and adjustments when needed!
        debug_assert!(new_entries.len() as i64 >= 0);
        let mut ordering = min_ordering.saturating_sub(new_entries.len() as i64);
        let created_at = UtcDateTimeMs::now();
        for entry in new_entries {
            let track_id = match &entry.item {
                Item::Separator(_) => None,
                Item::Track(TrackItem { uid }) => Some(self.resolve_track_id(uid)?),
            };
            let insertable = InsertableRecord::bind(id, track_id, ordering, &created_at, entry);
            let rows_affected = insertable
                .insert_into(playlist_entry::table)
                .execute(self.as_mut())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
            ordering = ordering.saturating_add(1);
        }
        Ok(())
    }

    fn remove_playlist_entries(
        &mut self,
        id: RecordId,
        index_range: &Range<usize>,
    ) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        if index_range.is_empty() {
            return Ok(0);
        }
        let offset = index_range.start as i64;
        debug_assert!(offset >= 0);
        let limit = index_range.len() as i64;
        debug_assert!(limit >= 0);
        let delete_row_ids_subselect = playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
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
                    .load::<i64>(self.as_mut())
                    .map_err(repo_error)?,
            ),
        );
        let rows_deleted: usize = diesel::delete(delete_target)
            .execute(self.as_mut())
            .map_err(repo_error)?;
        debug_assert!(rows_deleted <= index_range.len());
        Ok(rows_deleted)
    }

    fn remove_all_playlist_entries(&mut self, id: RecordId) -> RepoResult<usize> {
        use playlist_entry_db::schema::*;
        let rows_deleted: usize = diesel::delete(
            playlist_entry::table.filter(playlist_entry::playlist_id.eq(RowId::from(id))),
        )
        .execute(self.as_mut())
        .map_err(repo_error)?;
        Ok(rows_deleted)
    }

    fn reverse_all_playlist_entries(&mut self, id: RecordId) -> RepoResult<usize> {
        let min_ordering = min_playlist_entry_ordering(self, id)?;
        let max_ordering = max_playlist_entry_ordering(self, id)?;
        let (Some(min_ordering), Some(max_ordering)) = (min_ordering, max_ordering) else {
            debug_assert!(min_ordering.is_none());
            debug_assert!(max_ordering.is_none());
            debug_assert_eq!(self.count_playlist_entries(id).ok(), Some(0));
            return Ok(0);
        };
        reverse_playlist_entries(self, id, min_ordering, max_ordering)
    }

    fn insert_playlist_entries(
        &mut self,
        id: RecordId,
        before_index: usize,
        new_entries: &[Entry],
    ) -> RepoResult<()> {
        use playlist_entry_db::{models::*, schema::*};
        if new_entries.is_empty() {
            return Ok(());
        }
        let offset = before_index as i64;
        debug_assert!(offset >= 0);
        // The newly inserted entries will be assigned ordering numbers
        // from prev_ordering + 1 to prev_ordering + new_entries.len()
        let prev_ordering = if offset > 0 {
            playlist_entry::table
                .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
                .select(playlist_entry::ordering)
                .order_by(playlist_entry::ordering)
                .offset(offset - 1)
                .first::<i64>(self.as_mut())
                .optional()
                .map_err(repo_error)?
        } else {
            None
        };
        // Reordering is only needed if one or more entries follow the deleted range
        let next_ordering = playlist_entry::table
            .filter(playlist_entry::playlist_id.eq(RowId::from(id)))
            .select(playlist_entry::ordering)
            .order_by(playlist_entry::ordering)
            .offset(offset)
            .first::<i64>(self.as_mut())
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
                let rows_updated =
                    shift_playlist_entries_forward(self, id, next_ordering, delta_ordering)?;
                log::debug!(
                    "Reordered {rows_updated} entries of playlist {row_id} before inserting \
                     {num_new_entries} entries",
                    row_id = RowId::from(id),
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
        let created_at = UtcDateTimeMs::now();
        for entry in new_entries {
            let track_id = match &entry.item {
                Item::Separator(_) => None,
                Item::Track(TrackItem { uid }) => Some(self.resolve_track_id(uid)?),
            };
            let insertable = InsertableRecord::bind(id, track_id, ordering, &created_at, entry);
            let rows_affected = insertable
                .insert_into(playlist_entry::table)
                .execute(self.as_mut())
                .map_err(repo_error)?;
            debug_assert_eq!(1, rows_affected);
            ordering = ordering.saturating_add(1);
        }
        Ok(())
    }

    fn copy_all_playlist_entries(
        &mut self,
        source_id: RecordId,
        target_id: RecordId,
    ) -> RepoResult<usize> {
        use playlist_entry_db::{models::*, schema::*};
        let records = load_playlist_entry_records(self, source_id)?;
        let copied_count = records.len();
        let created_at = UtcDateTimeMs::now();
        for record in records {
            let (_id, ordering, track_id, entry) = record.into();
            let insertable =
                InsertableRecord::bind(target_id, track_id, ordering, &created_at, &entry);
            let rows_affected = insertable
                .insert_into(playlist_entry::table)
                .execute(self.as_mut())
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
