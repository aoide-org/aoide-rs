// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::Range;

use aoide_core::{
    playlist::{EntityHeader, EntityUid, Entry},
    util::clock::OffsetDateTimeMs,
};
use aoide_core_api::playlist::EntityWithEntriesSummary;
use aoide_repo::{
    playlist::{EntityRepo, EntryRepo, RecordHeader},
    prelude::RepoResult,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatchOperation {
    Append { entries: Vec<Entry> },
    Prepend { entries: Vec<Entry> },
    Insert { before: usize, entries: Vec<Entry> },
    CopyAll { source_playlist_uid: EntityUid },
    Move { range: Range<usize>, delta: isize },
    Remove { range: Range<usize> },
    RemoveAll,
    ReverseAll,
    ShuffleAll,
}

pub fn patch<Repo>(
    repo: &mut Repo,
    entity_header: &EntityHeader,
    operations: impl IntoIterator<Item = PatchOperation>,
) -> RepoResult<(RecordHeader, EntityWithEntriesSummary)>
where
    Repo: EntityRepo + EntryRepo,
{
    let updated_at = OffsetDateTimeMs::now_utc();
    let (record_header, next_rev) =
        repo.touch_playlist_entity_revision(entity_header, updated_at)?;
    for operation in operations {
        #[allow(clippy::enum_glob_use)]
        use PatchOperation::*;
        // TODO: Accept a streaming iterator that only borrows the items that it yields
        let operation = &operation;
        match operation {
            Append { entries } => {
                if entries.is_empty() {
                    continue;
                }
                repo.append_playlist_entries(record_header.id, entries)?;
            }
            Prepend { entries } => {
                if entries.is_empty() {
                    continue;
                }
                repo.prepend_playlist_entries(record_header.id, entries)?;
            }
            Insert { before, entries } => {
                if entries.is_empty() {
                    continue;
                }
                repo.insert_playlist_entries(record_header.id, *before, entries)?;
            }
            CopyAll {
                source_playlist_uid,
            } => {
                let source_playlist_id = repo.resolve_playlist_id(source_playlist_uid)?;
                repo.copy_all_playlist_entries(source_playlist_id, record_header.id)?;
            }
            Move { range, delta } => {
                if range.is_empty() || *delta == 0 {
                    continue;
                }
                repo.move_playlist_entries(record_header.id, range, *delta)?;
            }
            Remove { range } => {
                if range.is_empty() {
                    continue;
                }
                repo.remove_playlist_entries(record_header.id, range)?;
            }
            RemoveAll => {
                repo.remove_all_playlist_entries(record_header.id)?;
            }
            ReverseAll => {
                repo.reverse_all_playlist_entries(record_header.id)?;
            }
            ShuffleAll => {
                repo.shuffle_all_playlist_entries(record_header.id)?;
            }
        }
    }
    let (record_header, entity, entries) =
        repo.load_playlist_entity_with_entries_summary(record_header.id)?;
    debug_assert_eq!(next_rev, entity.hdr.rev);
    let entity_with_entries_summary = EntityWithEntriesSummary { entity, entries };
    Ok((record_header, entity_with_entries_summary))
}
