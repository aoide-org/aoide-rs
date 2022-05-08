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

use aoide_core::util::url::BaseUrl;
use num_derive::{FromPrimitive, ToPrimitive};

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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, ToPrimitive)]
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
    pub fn is_stale(self) -> bool {
        match self {
            Self::Outdated | Self::Added | Self::Modified => true,
            Self::Current | Self::Orphaned => false,
        }
    }

    /// Determine if an entry is stale and requires further processing.
    #[must_use]
    pub fn is_pending(self) -> bool {
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
