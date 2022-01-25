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

use url::Url;

#[cfg(feature = "backend")]
use aoide_core::util::url::{BaseUrl, BaseUrlError};

use crate::prelude::*;

mod _core {
    pub use aoide_core_api::media::tracker::*;
}

pub mod find_untracked_files;
pub mod import_files;
pub mod query_status;
pub mod scan_directories;
pub mod untrack_directories;

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FsTraversalParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<usize>,
}

#[cfg(feature = "frontend")]
impl From<_core::FsTraversalParams> for FsTraversalParams {
    fn from(from: _core::FsTraversalParams) -> Self {
        let _core::FsTraversalParams {
            root_url,
            max_depth,
        } = from;
        Self {
            root_url: root_url.map(Into::into),
            max_depth,
        }
    }
}

#[cfg(feature = "backend")]
impl TryFrom<FsTraversalParams> for _core::FsTraversalParams {
    type Error = BaseUrlError;

    fn try_from(from: FsTraversalParams) -> Result<Self, Self::Error> {
        let FsTraversalParams {
            root_url,
            max_depth,
        } = from;
        let root_url = root_url.map(BaseUrl::try_autocomplete_from).transpose()?;
        Ok(Self {
            root_url,
            max_depth,
        })
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(rename_all = "kebab-case")]
pub enum Progress {
    Idle,
    Scanning(FsTraversalProgress),
    Importing(ImportingProgress),
    FindingUntracked(FsTraversalProgress),
}

#[cfg(feature = "frontend")]
impl From<Progress> for _core::Progress {
    fn from(from: Progress) -> Self {
        use Progress::*;
        match from {
            Idle => Self::Idle,
            Scanning(progress) => Self::Scanning(progress.into()),
            Importing(progress) => Self::Importing(progress.into()),
            FindingUntracked(progress) => Self::FindingUntracked(progress.into()),
        }
    }
}

#[cfg(feature = "backend")]
impl From<_core::Progress> for Progress {
    fn from(from: _core::Progress) -> Self {
        use _core::Progress::*;
        match from {
            Idle => Self::Idle,
            Scanning(progress) => Self::Scanning(progress.into()),
            Importing(progress) => Self::Importing(progress.into()),
            FindingUntracked(progress) => Self::FindingUntracked(progress.into()),
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct FsTraversalProgress {
    pub entries: FsTraversalEntriesProgress,
    pub directories: FsTraversalDirectoriesProgress,
}

#[cfg(feature = "frontend")]
impl From<FsTraversalProgress> for _core::FsTraversalProgress {
    fn from(from: FsTraversalProgress) -> Self {
        let FsTraversalProgress {
            entries,
            directories,
        } = from;
        Self {
            entries: entries.into(),
            directories: directories.into(),
        }
    }
}

#[cfg(feature = "backend")]
impl From<_core::FsTraversalProgress> for FsTraversalProgress {
    fn from(from: _core::FsTraversalProgress) -> Self {
        let _core::FsTraversalProgress {
            entries,
            directories,
        } = from;
        Self {
            entries: entries.into(),
            directories: directories.into(),
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct FsTraversalEntriesProgress {
    pub skipped: usize,
    pub finished: usize,
}

#[cfg(feature = "frontend")]
impl From<FsTraversalEntriesProgress> for _core::FsTraversalEntriesProgress {
    fn from(from: FsTraversalEntriesProgress) -> Self {
        let FsTraversalEntriesProgress { skipped, finished } = from;
        Self { skipped, finished }
    }
}

#[cfg(feature = "backend")]
impl From<_core::FsTraversalEntriesProgress> for FsTraversalEntriesProgress {
    fn from(from: _core::FsTraversalEntriesProgress) -> Self {
        let _core::FsTraversalEntriesProgress { skipped, finished } = from;
        Self { skipped, finished }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct FsTraversalDirectoriesProgress {
    pub finished: usize,
}

#[cfg(feature = "frontend")]
impl From<FsTraversalDirectoriesProgress> for _core::FsTraversalDirectoriesProgress {
    fn from(from: FsTraversalDirectoriesProgress) -> Self {
        let FsTraversalDirectoriesProgress { finished } = from;
        Self { finished }
    }
}

#[cfg(feature = "backend")]
impl From<_core::FsTraversalDirectoriesProgress> for FsTraversalDirectoriesProgress {
    fn from(from: _core::FsTraversalDirectoriesProgress) -> Self {
        let _core::FsTraversalDirectoriesProgress { finished } = from;
        Self { finished }
    }
}

pub type ImportingProgress = import_files::Summary;

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(rename_all = "camelCase")]
pub enum Completion {
    Finished,
    Aborted,
}

#[cfg(feature = "frontend")]
impl From<Completion> for _core::Completion {
    fn from(from: Completion) -> Self {
        use Completion::*;
        match from {
            Finished => Self::Finished,
            Aborted => Self::Aborted,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_core::Completion> for Completion {
    fn from(from: _core::Completion) -> Self {
        use _core::Completion::*;
        match from {
            Finished => Self::Finished,
            Aborted => Self::Aborted,
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Status {
    pub directories: DirectoriesStatus,
}

#[cfg(feature = "backend")]
impl From<_core::Status> for Status {
    fn from(from: _core::Status) -> Self {
        let _core::Status { directories } = from;
        Self {
            directories: directories.into(),
        }
    }
}

#[cfg(feature = "frontend")]
impl From<Status> for _core::Status {
    fn from(from: Status) -> Self {
        let Status { directories } = from;
        Self {
            directories: directories.into(),
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DirectoriesStatus {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

#[cfg(feature = "backend")]
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

#[cfg(feature = "frontend")]
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum DirTrackingStatus {
    Current,
    Outdated,
    Added,
    Modified,
    Orphaned,
}

impl From<DirTrackingStatus> for _core::DirTrackingStatus {
    fn from(from: DirTrackingStatus) -> Self {
        use DirTrackingStatus::*;
        match from {
            Current => Self::Current,
            Outdated => Self::Outdated,
            Added => Self::Added,
            Modified => Self::Modified,
            Orphaned => Self::Orphaned,
        }
    }
}

impl From<_core::DirTrackingStatus> for DirTrackingStatus {
    fn from(from: _core::DirTrackingStatus) -> Self {
        use _core::DirTrackingStatus::*;
        match from {
            Current => Self::Current,
            Outdated => Self::Outdated,
            Added => Self::Added,
            Modified => Self::Modified,
            Orphaned => Self::Orphaned,
        }
    }
}
