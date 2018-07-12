// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::fmt;
use std::ops::Deref;

///////////////////////////////////////////////////////////////////////
/// SampleLayout
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
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
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
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

pub type NumberOfSamples = f64;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SampleLength(pub NumberOfSamples);

impl Deref for SampleLength {
    type Target = NumberOfSamples;

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
