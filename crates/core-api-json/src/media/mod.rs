// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

pub mod source;
pub mod tracker;

mod _core {
    pub(super) use aoide_core_api::media::*;
}

/// Controls when metadata is (re-)imported from the media sources.
///
/// * once: Only import metadata once, never re-import.
/// * modified (recommended): Only (re-)import metadata from media source if modified and if the
///   current track revision matches the synchronized revision.
/// * modified-resync: Only (re-)import metadata from media source if modified but regardless of
///   the synchronized revision, i.e. allow to overwrite changed metadata with metadata imported
///   from the media source to enforce resynchronization. Use this variant if the metadata stored
///   in file tags is complete and the primary metadata source for tracks, i.e. if the file system
///   is considered as the actual "book of records".
/// * always: Always (re-)import metadata from media source, regardless of modification time and
///   synchronization status.
#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "MediaSyncMode")
)]
#[serde(rename_all = "kebab-case")]
pub enum SyncMode {
    Once,
    Modified,
    ModifiedResync,
    Always,
}

#[cfg(feature = "backend")]
impl From<SyncMode> for _core::SyncMode {
    fn from(from: SyncMode) -> Self {
        use SyncMode as From;
        match from {
            From::Once => Self::Once,
            From::Modified => Self::Modified,
            From::ModifiedResync => Self::ModifiedResync,
            From::Always => Self::Always,
        }
    }
}

#[cfg(feature = "frontend")]
impl From<_core::SyncMode> for SyncMode {
    fn from(from: _core::SyncMode) -> Self {
        use _core::SyncMode as From;
        match from {
            From::Once => Self::Once,
            From::Modified => Self::Modified,
            From::ModifiedResync => Self::ModifiedResync,
            From::Always => Self::Always,
        }
    }
}
