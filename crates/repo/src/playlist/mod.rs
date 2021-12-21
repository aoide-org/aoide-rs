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

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

use std::ops::Range;

use crate::{collection::RecordId as CollectionId, prelude::*};

use aoide_core::{playlist::*, util::clock::DateTime};

use rand::{seq::SliceRandom, thread_rng};

pub trait Repo: EntityRepo + EntryRepo {
    fn load_playlist_entity_with_entries(&self, id: RecordId) -> RepoResult<EntityWithEntries>;

    fn load_collected_playlist_entities_with_entries_summary(
        &self,
        collection_id: CollectionId,
        kind: Option<&str>,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = (Entity, EntriesSummary),
        >,
    ) -> RepoResult<()>;

    fn load_playlist_entity_with_entries_summary(
        &self,
        playlist_id: RecordId,
    ) -> RepoResult<(RecordHeader, Entity, EntriesSummary)> {
        let (record_header, entity) = self.load_playlist_entity(playlist_id)?;
        let entries_summary = self.load_playlist_entries_summary(playlist_id)?;
        Ok((record_header, entity, entries_summary))
    }
}

pub trait EntityRepo {
    entity_repo_trait_common_functions!(RecordId, Entity, Playlist);

    fn insert_collected_playlist_entity(
        &self,
        collection_id: CollectionId,
        created_at: DateTime,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;
}

pub fn prepend_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
    new_entries: &[Entry],
) -> RepoResult<()> {
    entry_repo.insert_playlist_entries(playlist_id, 0, new_entries)
}

pub fn append_playlist_entries_default<R: EntryRepo + ?Sized>(
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

pub fn move_playlist_entries_default<R: EntryRepo + ?Sized>(
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
    let _removed_count = entry_repo.remove_playlist_entries(playlist_id, index_range)?;
    debug_assert_eq!(_removed_count, index_range.len());
    let insert_index = if delta_index > 0 {
        (index_range.start + delta_index as usize)
            .min(entry_repo.count_playlist_entries(playlist_id)?)
    } else {
        debug_assert!(delta_index < 0);
        index_range.start - (-delta_index as usize).min(index_range.start)
    };
    entry_repo.insert_playlist_entries(playlist_id, insert_index, &moved_entries)
}

pub fn remove_all_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
) -> RepoResult<usize> {
    let entries_count = entry_repo.count_playlist_entries(playlist_id)?;
    if entries_count == 0 {
        return Ok(entries_count);
    }
    entry_repo.remove_playlist_entries(playlist_id, &(0..entries_count))
}

pub fn shuffle_all_playlist_entries_default<R: EntryRepo + ?Sized>(
    entry_repo: &R,
    playlist_id: RecordId,
) -> RepoResult<()> {
    let mut entries = entry_repo.load_all_playlist_entries(playlist_id)?;
    entries.shuffle(&mut thread_rng());
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
