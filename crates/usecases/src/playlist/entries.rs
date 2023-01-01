// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::playlist::{EntityUid, Entry};

use std::ops::Range;

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
