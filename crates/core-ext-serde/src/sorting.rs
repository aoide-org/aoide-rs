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

mod _inner {
    pub use crate::_inner::sorting::*;
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum SortDirection {
    #[serde(rename = "asc")]
    Ascending,

    #[serde(rename = "desc")]
    Descending,
}

impl From<SortDirection> for _inner::SortDirection {
    fn from(from: SortDirection) -> Self {
        use SortDirection::*;
        match from {
            Ascending => Self::Ascending,
            Descending => Self::Descending,
        }
    }
}

impl From<_inner::SortDirection> for SortDirection {
    fn from(from: _inner::SortDirection) -> Self {
        use _inner::SortDirection::*;
        match from {
            Ascending => Self::Ascending,
            Descending => Self::Descending,
        }
    }
}
