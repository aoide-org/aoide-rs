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

mod _core {
    pub use aoide_core::audio::signal::*;
}

///////////////////////////////////////////////////////////////////////
// BitRate
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct BitRateBps(_core::BitsPerSecond);

impl From<_core::BitRateBps> for BitRateBps {
    fn from(from: _core::BitRateBps) -> Self {
        Self(from.0)
    }
}

impl From<BitRateBps> for _core::BitRateBps {
    fn from(from: BitRateBps) -> Self {
        Self(from.0)
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SampleRateHz(_core::SamplesPerSecond);

impl From<_core::SampleRateHz> for SampleRateHz {
    fn from(from: _core::SampleRateHz) -> Self {
        Self(from.0)
    }
}

impl From<SampleRateHz> for _core::SampleRateHz {
    fn from(from: SampleRateHz) -> Self {
        Self(from.0)
    }
}

///////////////////////////////////////////////////////////////////////
// Loudness
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LoudnessLufs(_core::LufsValue);

impl From<_core::LoudnessLufs> for LoudnessLufs {
    fn from(from: _core::LoudnessLufs) -> Self {
        Self(from.0)
    }
}

impl From<LoudnessLufs> for _core::LoudnessLufs {
    fn from(from: LoudnessLufs) -> Self {
        Self(from.0)
    }
}
