// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Range;

use rand::seq::SliceRandom as _;

use aoide_core::{
    playlist::{Entity, EntityHeader, EntityUid, EntityWithEntries, EntriesSummary, Entry},
    util::{clock::DateTime, random::adhoc_rng},
};

use aoide_core_api::playlist::EntityWithEntriesSummary;

use crate::{collection::RecordId as CollectionId, prelude::*};

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

pub trait EntityRepo: EntryRepo {
    entity_repo_trait_common_functions!(RecordId, Entity, EntityUid, EntityHeader, Playlist);

    fn load_playlist_entity_with_entries(&self, id: RecordId) -> RepoResult<EntityWithEntries>;

    fn load_playlist_entity_with_entries_summary(
        &self,
        playlist_id: RecordId,
    ) -> RepoResult<(RecordHeader, Entity, EntriesSummary)> {
        let (record_header, entity) = self.load_playlist_entity(playlist_id)?;
        let entries_summary = self.load_playlist_entries_summary(playlist_id)?;
        Ok((record_header, entity, entries_summary))
    }
}

pub trait CollectionRepo {
    fn insert_playlist_entity(
        &self,
        collection_id: CollectionId,
        created_at: DateTime,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;

    fn load_playlist_entities_with_entries_summary(
        &self,
        collection_id: CollectionId,
        kind: Option<&str>,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = EntityWithEntriesSummary,
        >,
    ) -> RepoResult<()>;
}

/// Prepend playlist entries by insertion
///
/// This default implementation works but is probably inefficient.
fn prepend_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
    new_entries: &[Entry],
) -> RepoResult<()> {
    entry_repo.insert_playlist_entries(playlist_id, 0, new_entries)
}

/// Append playlist entries by insertion
///
/// This default implementation works but is probably inefficient.
fn append_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
    new_entries: &[Entry],
) -> RepoResult<()> {
    if new_entries.is_empty() {
        return Ok(());
    }
    let entries_count = entry_repo.count_playlist_entries(playlist_id)?;
    entry_repo.insert_playlist_entries(playlist_id, entries_count, new_entries)
}

/// Move playlist entries by first removing and then reinserting the given range
///
/// This default implementation works but is probably inefficient.
fn move_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
    index_range: &Range<usize>,
    delta_index: isize,
) -> RepoResult<()> {
    if index_range.is_empty() || delta_index == 0 {
        return Ok(());
    }
    let mut moved_entries = entry_repo.load_all_playlist_entries(playlist_id)?;
    moved_entries.truncate(index_range.end.min(moved_entries.len()));
    moved_entries.drain(0..index_range.start.min(moved_entries.len()));
    debug_assert_eq!(moved_entries.len(), index_range.len());
    let removed_count = entry_repo.remove_playlist_entries(playlist_id, index_range)?;
    debug_assert_eq!(removed_count, index_range.len());
    let insert_index = if delta_index > 0 {
        (index_range.start + delta_index as usize)
            .min(entry_repo.count_playlist_entries(playlist_id)?)
    } else {
        debug_assert!(delta_index < 0);
        index_range.start - (-delta_index as usize).min(index_range.start)
    };
    entry_repo.insert_playlist_entries(playlist_id, insert_index, &moved_entries)
}

/// Remove all playlist entries one by one
///
/// This default implementation works but is probably inefficient.
fn remove_all_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
) -> RepoResult<usize> {
    let entries_count = entry_repo.count_playlist_entries(playlist_id)?;
    if entries_count == 0 {
        return Ok(entries_count);
    }
    entry_repo.remove_playlist_entries(playlist_id, &(0..entries_count))
}

/// Shuffle playlist by first removing and then reinserting all entries
///
/// This default implementation works but is probably inefficient.
fn shuffle_all_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
) -> RepoResult<()> {
    let mut entries = entry_repo.load_all_playlist_entries(playlist_id)?;
    entries.shuffle(&mut adhoc_rng() as _);
    entry_repo.remove_all_playlist_entries(playlist_id)?;
    entry_repo.append_playlist_entries(playlist_id, &entries)?;
    Ok(())
}

pub trait EntryRepo {
    fn insert_playlist_entries(
        &self,
        playlist_id: RecordId,
        before_index: usize,
        new_entries: &[Entry],
    ) -> RepoResult<()>;

    fn remove_playlist_entries(
        &self,
        playlist_id: RecordId,
        index_range: &Range<usize>,
    ) -> RepoResult<usize>;

    fn prepend_playlist_entries(
        &self,
        playlist_id: RecordId,
        new_entries: &[Entry],
    ) -> RepoResult<()> {
        prepend_playlist_entries_default(self, playlist_id, new_entries)
    }

    fn append_playlist_entries(
        &self,
        playlist_id: RecordId,
        new_entries: &[Entry],
    ) -> RepoResult<()> {
        append_playlist_entries_default(self, playlist_id, new_entries)
    }

    fn move_playlist_entries(
        &self,
        playlist_id: RecordId,
        index_range: &Range<usize>,
        delta_index: isize,
    ) -> RepoResult<()> {
        move_playlist_entries_default(self, playlist_id, index_range, delta_index)
    }

    fn remove_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize> {
        remove_all_playlist_entries_default(self, playlist_id)
    }

    fn reverse_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize>;

    fn shuffle_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<()> {
        shuffle_all_playlist_entries_default(self, playlist_id)
    }

    /// Copy all entries from the source playlist into the target playlist.
    ///
    /// The order among the copied entries is preserved. If the target playlist
    /// already contains entries copying may fail and the ordering of existing
    /// and copied entries is undefined.
    fn copy_all_playlist_entries(
        &self,
        source_playlist_id: RecordId,
        target_playlist_id: RecordId,
    ) -> RepoResult<usize>;

    fn count_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize>;

    fn load_all_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<Vec<Entry>>;

    fn load_playlist_entries_summary(&self, playlist_id: RecordId) -> RepoResult<EntriesSummary>;
}

#[derive(Debug, Default)]
pub struct EntityWithEntriesSummaryCollector(Vec<EntityWithEntriesSummary>);

impl EntityWithEntriesSummaryCollector {
    #[must_use]
    pub const fn new(inner: Vec<EntityWithEntriesSummary>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn finish(self) -> Vec<EntityWithEntriesSummary> {
        let Self(inner) = self;
        inner
    }
}

impl RecordCollector for EntityWithEntriesSummaryCollector {
    type Header = RecordHeader;
    type Record = EntityWithEntriesSummary;

    fn collect(&mut self, _header: RecordHeader, record: EntityWithEntriesSummary) {
        let Self(inner) = self;
        inner.push(record);
    }
}

impl ReservableRecordCollector for EntityWithEntriesSummaryCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
