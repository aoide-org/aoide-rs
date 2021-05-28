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
