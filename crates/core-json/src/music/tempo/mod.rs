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
    pub use aoide_core::music::tempo::*;
}

///////////////////////////////////////////////////////////////////////
// Tempo
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct TempoBpm(_core::Bpm);

impl From<_core::TempoBpm> for TempoBpm {
    fn from(from: _core::TempoBpm) -> Self {
        Self(from.to_raw())
    }
}

impl From<TempoBpm> for _core::TempoBpm {
    fn from(from: TempoBpm) -> Self {
        Self::from_raw(from.0)
    }
}
