// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod source;
pub mod tracker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    /// Only import metadata once, never re-import.
    Once,

    /// Only (re-)import metadata from media source if modified
    /// and if the current track revision matches the synchronized
    /// revision.
    Modified,

    /// Only (re-)import metadata from media source if modified
    /// but regardless of the synchronized revision, i.e. allow to
    /// overwrite changed metadata with metadata imported from the
    /// media source for resynchronization.
    ModifiedResync,

    /// Always (re-)import metadata from media source, regardless
    /// of modification time and synchronization status.
    Always,
}
