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

pub mod import;
pub mod scan;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Completion {
    Finished,
    Aborted,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Status {
    pub directories: DirectoriesStatus,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DirectoriesStatus {
    pub current: usize,
    pub outdated: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Progress {
    Idle,
    Scanning(ScanningProgress),
    Importing(ImportingProgress),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanningProgress {
    pub entries: ScanningEntriesProgress,
    pub directories: ScanningDirectoriesProgress,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanningEntriesProgress {
    pub skipped: usize,
    pub finished: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanningDirectoriesProgress {
    pub finished: usize,
}

pub type ImportingProgress = import::Summary;
