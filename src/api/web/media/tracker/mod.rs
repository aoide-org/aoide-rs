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
    entries_skipped: usize,
    entries_finished: usize,
    directories_finished: usize,
}

pub type ImportingProgress = import::Summary;

impl From<digest::Progress> for HashingProgress {
    fn from(from: digest::Progress) -> Self {
        let digest::Progress {
            entries_skipped,
            entries_finished,
            directories_finished,
        } = from;
        Self {
            entries_skipped,
            entries_finished,
            directories_finished,
        }
    }
}
