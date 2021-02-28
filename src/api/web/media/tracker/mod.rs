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

///////////////////////////////////////////////////////////////////////

use super::*;

use aoide_media::fs::digest;

pub mod hash;
pub mod import;
pub mod query_status;
pub mod untrack;

mod uc {
    pub use crate::usecases::media::tracker::*;
    pub use aoide_usecases::media::tracker::*;
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Completion {
    Finished,
    Aborted,
}

impl From<uc::Completion> for Completion {
    fn from(from: uc::Completion) -> Self {
        use uc::Completion::*;
        match from {
            Finished => Self::Finished,
            Aborted => Self::Aborted,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Progress {
    Idle,
    Hashing(HashingProgress),
    Importing(ImportingProgress),
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashingProgress {
    entries: HashingEntriesProgress,
    directories: HashingDirectoriesProgress,
}

impl From<digest::Progress> for HashingProgress {
    fn from(from: digest::Progress) -> Self {
        let digest::Progress {
            entries,
            directories,
        } = from;
        Self {
            entries: entries.into(),
            directories: directories.into(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashingEntriesProgress {
    skipped: usize,
    finished: usize,
}

impl From<digest::EntriesProgress> for HashingEntriesProgress {
    fn from(from: digest::EntriesProgress) -> Self {
        let digest::EntriesProgress { skipped, finished } = from;
        Self { skipped, finished }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HashingDirectoriesProgress {
    finished: usize,
}

impl From<digest::DirectoriesProgress> for HashingDirectoriesProgress {
    fn from(from: digest::DirectoriesProgress) -> Self {
        let digest::DirectoriesProgress { finished } = from;
        Self { finished }
    }
}

pub type ImportingProgress = import::Summary;
