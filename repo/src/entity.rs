// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use aoide_core::entity::EntityUid;

pub trait Repo {
    fn resolve_repo_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>>;
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntityDataFormat {
    JSON = 1,
}

impl fmt::Display for EntityDataFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EntityDataFormat::JSON => f.write_str("JSON"),
        }
    }
}

pub type EntityDataVersionNumber = u16;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct EntityDataVersion {
    pub major: EntityDataVersionNumber,
    pub minor: EntityDataVersionNumber,
}

impl EntityDataVersion {
    pub fn next_major(self) -> Self {
        Self {
            major: self.major + 1,
            minor: 0,
        }
    }

    pub fn next_minor(self) -> Self {
        Self {
            major: self.major,
            minor: self.minor + 1,
        }
    }
}

impl fmt::Display for EntityDataVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

pub type EntityBodyData = (EntityDataFormat, EntityDataVersion, Vec<u8>);

pub type EntityData = (EntityHeader, EntityBodyData);
