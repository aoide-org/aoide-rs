// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub mod channel;
pub mod sample;
pub mod signal;

mod _core {
    pub(super) use aoide_core::audio::*;
}

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(PartialEq))]
pub struct PositionMs(_core::PositionInMilliseconds);

impl From<_core::PositionMs> for PositionMs {
    fn from(from: _core::PositionMs) -> Self {
        let _core::PositionMs(ms) = from;
        Self(ms)
    }
}

impl From<PositionMs> for _core::PositionMs {
    fn from(from: PositionMs) -> Self {
        let PositionMs(ms) = from;
        Self(ms)
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(PartialEq))]
pub struct DurationMs(_core::DurationInMilliseconds);

impl From<_core::DurationMs> for DurationMs {
    fn from(from: _core::DurationMs) -> Self {
        Self(from.to_inner())
    }
}

impl From<DurationMs> for _core::DurationMs {
    fn from(from: DurationMs) -> Self {
        let DurationMs(ms) = from;
        Self::from_inner(ms)
    }
}
