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

pub trait EntryRepo {
    fn prepend_playlist_entries(
        &self,
        playlist_id: RecordId,
        new_entries: Vec<Entry>,
    ) -> RepoResult<()> {
        self.insert_playlist_entries(playlist_id, 0, new_entries)
    }

    fn append_playlist_entries(
        &self,
        playlist_id: RecordId,
        new_entries: Vec<Entry>,
    ) -> RepoResult<()> {
        if new_entries.is_empty() {
            return Ok(());
        }
        let EntriesSummary {
            total_count: entries_count,
            ..
        } = self.load_playlist_entries_summary(playlist_id)?;
        self.insert_playlist_entries(playlist_id, entries_count, new_entries)
    }

    fn move_playlist_entries(
        &self,
        playlist_id: RecordId,
        index_range: &Range<usize>,
        delta_index: isize,
    ) -> RepoResult<()> {
        if index_range.is_empty() || delta_index == 0 {
            return Ok(());
        }
        let mut moved_entries = self.load_playlist_entries(playlist_id)?;
        moved_entries.truncate(index_range.end.min(moved_entries.len()));
        moved_entries.drain(0..index_range.start.min(moved_entries.len()));
        let _removed_count = self.remove_playlist_entries(playlist_id, index_range)?;
        debug_assert_eq!(_removed_count, moved_entries.len());
        let insert_index = if delta_index > 0 {
            index_range.start + delta_index as usize
        } else {
            debug_assert!(delta_index < 0);
            index_range.start - (-delta_index as usize).min(index_range.start)
        };
        self.insert_playlist_entries(playlist_id, insert_index, moved_entries)
    }

    fn remove_playlist_entries(
        &self,
        playlist_id: RecordId,
        index_range: &Range<usize>,
    ) -> RepoResult<usize>;

    fn clear_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize> {
        let EntriesSummary {
            total_count: entries_count,
            ..
        } = self.load_playlist_entries_summary(playlist_id)?;
        if entries_count == 0 {
            return Ok(entries_count);
        }
        self.remove_playlist_entries(playlist_id, &(0..entries_count))
    }

    fn reverse_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<usize>;

    fn insert_playlist_entries(
        &self,
        playlist_id: RecordId,
        before_index: usize,
        new_entries: Vec<Entry>,
    ) -> RepoResult<()>;

    fn shuffle_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<()>;

    fn load_playlist_entries(&self, playlist_id: RecordId) -> RepoResult<Vec<Entry>>;

    fn load_playlist_entries_summary(&self, playlist_id: RecordId) -> RepoResult<EntriesSummary>;

    fn delete_playlist_entries_with_tracks_from_other_collections(&self) -> RepoResult<usize>;
}
