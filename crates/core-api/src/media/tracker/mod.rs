// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::util::url::BaseUrl;
use strum::FromRepr;

pub mod find_untracked_files;
pub mod import_files;
pub mod query_status;
pub mod scan_directories;
pub mod untrack_directories;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FsTraversalParams {
    pub root_url: Option<BaseUrl>,
    pub max_depth: Option<usize>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromRepr)]
#[repr(u8)]
pub enum DirTrackingStatus {
    Current = 0,
    Outdated = 1,
    Added = 2,
    Modified = 3,
    Orphaned = 4,
}

impl DirTrackingStatus {
    /// Determine if an entry is stale.
    #[must_use]
    pub const fn is_stale(self) -> bool {
        match self {
            Self::Outdated | Self::Added | Self::Modified => true,
            Self::Current | Self::Orphaned => false,
        }
    }

    /// Determine if an entry is stale and requires further processing.
    #[must_use]
    pub const fn is_pending(self) -> bool {
        match self {
            Self::Added | Self::Modified => {
                debug_assert!(self.is_stale());
                true
            }
            Self::Current | Self::Outdated | Self::Orphaned => false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    Finished,
    Aborted,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub directories: DirectoriesStatus,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DirectoriesStatus {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Progress {
    Idle,
    Scanning(FsTraversalProgress),
    Importing(ImportingProgress),
    FindingUntracked(FsTraversalProgress),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FsTraversalProgress {
    pub entries: FsTraversalEntriesProgress,
    pub directories: FsTraversalDirectoriesProgress,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FsTraversalEntriesProgress {
    pub skipped: usize,
    pub finished: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FsTraversalDirectoriesProgress {
    pub finished: usize,
}

pub type ImportingProgress = import_files::Summary;
