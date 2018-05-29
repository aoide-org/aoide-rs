// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

#[cfg(test)]
mod tests;

use std::u32;
use std::fmt;
use std::ops::Deref;

///////////////////////////////////////////////////////////////////////
/// SampleLayout
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum SampleLayout {
    // Samples grouped by channel
    // Example for stereo signal with channels L+R: [LLLL|RRRR]
    Planar,

    // Samples grouped by frame
    // Example for stereo signal with channels L+R: [LR|LR|LR|LR]
    Interleaved,
}

impl fmt::Display for SampleLayout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleFormat
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
pub enum SampleFormat {
    Float32,
}

impl fmt::Display for SampleFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleType
///////////////////////////////////////////////////////////////////////

pub type SampleType = f32;

///////////////////////////////////////////////////////////////////////
/// SamplePosition
///////////////////////////////////////////////////////////////////////

pub type SamplePositionType = f64;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SamplePosition(pub SamplePositionType);

impl Deref for SamplePosition {
    type Target = SamplePositionType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SamplePosition {
    pub fn is_valid(self) -> bool {
        self.is_finite()
    }

    pub fn is_integer(self) -> bool {
        self.trunc() == *self
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleLength
///////////////////////////////////////////////////////////////////////

pub type SampleLengthType = f64;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SampleLength(pub SampleLengthType);

impl Deref for SampleLength {
    type Target = SampleLengthType;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SampleLength {
    pub fn is_valid(self) -> bool {
        self.is_finite() && self.is_sign_positive()
    }

    pub fn is_integer(self) -> bool {
        self.trunc() == *self
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleRange
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SampleRange {
    pub pos: SamplePosition,
    pub len: SampleLength,
}

impl SampleRange {
    pub fn is_valid(&self) -> bool {
        self.pos.is_valid() && self.len.is_valid()
    }

    pub fn is_integer(&self) -> bool {
        self.pos.is_integer() && self.len.is_integer()
    }

    pub fn start(&self) -> SamplePosition {
        self.pos
    }

    pub fn end(&self) -> SamplePosition {
        SamplePosition(*self.pos + *self.len)
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SamplesPerSecond = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SampleRate {
    pub hz: SamplesPerSecond,
}

impl SampleRate {
    pub const UNIT_OF_MEASURE: &'static str = "Hz";

    pub const MAX: Self = SampleRate { hz: u32::MAX };

    pub const COMPACT_DISC: Self = SampleRate { hz: 44_100 };
    pub const STUDIO_48KHZ: Self = SampleRate { hz: 48_000 };
    pub const STUDIO_96KHZ: Self = SampleRate { hz: 96_000 };
    pub const STUDIO_192KHZ: Self = SampleRate { hz: 192_000 };

    pub fn hz(hz: SamplesPerSecond) -> Self {
        Self { hz }
    }

    pub fn is_valid(&self) -> bool {
        self.hz > 0
    }
}

impl fmt::Display for SampleRate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.hz, SampleRate::UNIT_OF_MEASURE)
    }
}
