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

use crate::prelude::*;

mod _core {
    pub use aoide_core::usecases::media::tracker::*;
}

pub mod import;
pub mod scan;
pub mod untrack;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Progress {
    Idle,
    Scanning(ScanningProgress),
    Importing(ImportingProgress),
}

impl From<Progress> for _core::Progress {
    fn from(from: Progress) -> Self {
        use Progress::*;
        match from {
            Idle => Self::Idle,
            Scanning(progress) => Self::Scanning(progress.into()),
            Importing(progress) => Self::Importing(progress.into()),
        }
    }
}

impl From<_core::Progress> for Progress {
    fn from(from: _core::Progress) -> Self {
        use _core::Progress::*;
        match from {
            Idle => Self::Idle,
            Scanning(progress) => Self::Scanning(progress.into()),
            Importing(progress) => Self::Importing(progress.into()),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanningProgress {
    pub entries: ScanningEntriesProgress,
    pub directories: ScanningDirectoriesProgress,
}

impl From<ScanningProgress> for _core::ScanningProgress {
    fn from(from: ScanningProgress) -> Self {
        let ScanningProgress {
            entries,
            directories,
        } = from;
        Self {
            entries: entries.into(),
            directories: directories.into(),
        }
    }
}

impl From<_core::ScanningProgress> for ScanningProgress {
    fn from(from: _core::ScanningProgress) -> Self {
        let _core::ScanningProgress {
            entries,
            directories,
        } = from;
        Self {
            entries: entries.into(),
            directories: directories.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanningEntriesProgress {
    pub skipped: usize,
    pub finished: usize,
}

impl From<ScanningEntriesProgress> for _core::ScanningEntriesProgress {
    fn from(from: ScanningEntriesProgress) -> Self {
        let ScanningEntriesProgress { skipped, finished } = from;
        Self { skipped, finished }
    }
}

impl From<_core::ScanningEntriesProgress> for ScanningEntriesProgress {
    fn from(from: _core::ScanningEntriesProgress) -> Self {
        let _core::ScanningEntriesProgress { skipped, finished } = from;
        Self { skipped, finished }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanningDirectoriesProgress {
    pub finished: usize,
}

impl From<ScanningDirectoriesProgress> for _core::ScanningDirectoriesProgress {
    fn from(from: ScanningDirectoriesProgress) -> Self {
        let ScanningDirectoriesProgress { finished } = from;
        Self { finished }
    }
}

impl From<_core::ScanningDirectoriesProgress> for ScanningDirectoriesProgress {
    fn from(from: _core::ScanningDirectoriesProgress) -> Self {
        let _core::ScanningDirectoriesProgress { finished } = from;
        Self { finished }
    }
}

pub type ImportingProgress = import::Summary;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Completion {
    Finished,
    Aborted,
}

impl From<Completion> for _core::Completion {
    fn from(from: Completion) -> Self {
        use Completion::*;
        match from {
            Finished => Self::Finished,
            Aborted => Self::Aborted,
        }
    }
}

impl From<_core::Completion> for Completion {
    fn from(from: _core::Completion) -> Self {
        use _core::Completion::*;
        match from {
            Finished => Self::Finished,
            Aborted => Self::Aborted,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Status {
    pub directories: DirectoriesStatus,
}

impl From<_core::Status> for Status {
    fn from(from: _core::Status) -> Self {
        let _core::Status { directories } = from;
        Self {
            directories: directories.into(),
        }
    }
}

impl From<Status> for _core::Status {
    fn from(from: Status) -> Self {
        let Status { directories } = from;
        Self {
            directories: directories.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DirectoriesStatus {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

impl From<_core::DirectoriesStatus> for DirectoriesStatus {
    fn from(from: _core::DirectoriesStatus) -> Self {
        let _core::DirectoriesStatus {
            current,
            outdated,
            added,
            modified,
            orphaned,
        } = from;
        Self {
            current,
            outdated,
            added,
            modified,
            orphaned,
        }
    }
}

impl From<DirectoriesStatus> for _core::DirectoriesStatus {
    fn from(from: DirectoriesStatus) -> Self {
        let DirectoriesStatus {
            current,
            outdated,
            added,
            modified,
            orphaned,
        } = from;
        Self {
            current,
            outdated,
            added,
            modified,
            orphaned,
        }
    }
}
