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
    pub use aoide_core::audio::sample::*;
}

///////////////////////////////////////////////////////////////////////
// SamplePosition
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq))]
pub struct SamplePosition(_core::SamplePositionType);

impl From<_core::SamplePosition> for SamplePosition {
    fn from(from: _core::SamplePosition) -> Self {
        let _core::SamplePosition(sample) = from;
        Self(sample)
    }
}

impl From<SamplePosition> for _core::SamplePosition {
    fn from(from: SamplePosition) -> Self {
        let SamplePosition(sample) = from;
        Self(sample)
    }
}
