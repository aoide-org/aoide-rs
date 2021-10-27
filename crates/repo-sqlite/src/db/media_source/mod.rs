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

pub mod models;
pub mod schema;
pub mod subselect;

use aoide_repo::media::source::RecordHeader;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i16)]
enum ArtworkSource {
    Missing = 0,
    Embedded = 1,
    Linked = 2,
}

impl ArtworkSource {
    pub fn try_read(value: i16) -> Option<Self> {
        let read = match value {
            0 => Self::Missing,
            1 => Self::Embedded,
            2 => Self::Linked,
            _ => return None,
        };
        Some(read)
    }

    pub const fn write(self) -> i16 {
        self as i16
    }
}
