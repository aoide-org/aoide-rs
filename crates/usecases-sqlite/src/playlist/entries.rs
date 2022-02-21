// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::util::clock::DateTime;
use aoide_core_api::playlist::EntityWithEntriesSummary;

use std::ops::Range;

#[derive(Debug, Clone, Eq, PartialEq)]
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

pub fn patch(
    connection: &SqliteConnection,
    entity_header: &EntityHeader,
    operations: impl IntoIterator<Item = PatchOperation>,
) -> Result<(RecordHeader, EntityWithEntriesSummary)> {
    let updated_at = DateTime::now_utc();
    let repo = RepoConnection::new(connection);
    let (record_header, _next_rev) =
        repo.touch_playlist_entity_revision(entity_header, updated_at)?;
    for operation in operations {
        use PatchOperation::*;
        match operation {
            Append { entries } => {
                if entries.is_empty() {
                    continue;
                }
                repo.append_playlist_entries(record_header.id, &entries)?;
            }
            Prepend { entries } => {
                if entries.is_empty() {
                    continue;
                }
                repo.prepend_playlist_entries(record_header.id, &entries)?;
            }
            Insert { before, entries } => {
                if entries.is_empty() {
                    continue;
                }
                repo.insert_playlist_entries(record_header.id, before, &entries)?;
            }
            CopyAll {
                source_playlist_uid,
            } => {
                let source_playlist_id = repo.resolve_playlist_id(&source_playlist_uid)?;
                repo.copy_all_playlist_entries(source_playlist_id, record_header.id)?;
            }
            Move { range, delta } => {
                if range.is_empty() || delta == 0 {
                    continue;
                }
                repo.move_playlist_entries(record_header.id, &range, delta)?;
            }
            Remove { range } => {
                if range.is_empty() {
                    continue;
                }
                repo.remove_playlist_entries(record_header.id, &range)?;
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
    debug_assert_eq!(_next_rev, entity.hdr.rev);
    let entity_with_entries_summary = EntityWithEntriesSummary { entity, entries };
    Ok((record_header, entity_with_entries_summary))
}
