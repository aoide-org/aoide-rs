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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Progress {
    Idle,
    Scanning(ScanningProgress),
    Importing(ImportingProgress),
}

impl From<Progress> for _core::Progress {
    fn from(from: Progress) -> Self {
        match from {
            Progress::Idle => Self::Idle,
            Progress::Scanning(progress) => Self::Scanning(progress.into()),
            Progress::Importing(progress) => Self::Importing(progress.into()),
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

pub type ImportingProgress = import::Summary;
