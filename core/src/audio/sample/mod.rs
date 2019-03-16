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

use std::fmt;

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

impl From<SamplePositionType> for SamplePosition {
    fn from(from: SamplePositionType) -> Self {
        Self(from)
    }
}

impl From<SamplePosition> for SamplePositionType {
    fn from(from: SamplePosition) -> Self {
        from.0
    }
}

impl IsValid for SamplePosition {
    fn is_valid(&self) -> bool {
        self.0.is_finite()
    }
}

impl IsInteger for SamplePosition {
    fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleLength
///////////////////////////////////////////////////////////////////////

pub type NumberOfSamples = f64;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SampleLength(pub NumberOfSamples);

impl From<NumberOfSamples> for SampleLength {
    fn from(from: NumberOfSamples) -> Self {
        Self(from)
    }
}

impl From<SampleLength> for NumberOfSamples {
    fn from(from: SampleLength) -> Self {
        from.0
    }
}

impl IsValid for SampleLength {
    fn is_valid(&self) -> bool {
        self.0.is_finite() && self.0.is_sign_positive()
    }
}

impl IsInteger for SampleLength {
    fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleRange
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SampleRange {
    pub start: SamplePosition,
    pub end: SamplePosition,
}

impl SampleRange {
    pub fn reverse(self) -> Self {
        Self {
            start: self.end,
            end: self.start,
        }
    }

    pub fn is_forward(&self) -> bool {
        self.start < self.end
    }

    pub fn is_backward(&self) -> bool {
        self.start > self.end
    }

    pub fn length(&self) -> SampleLength {
        SampleLength((self.end.0 - self.start.0).abs())
    }
}

impl IsValid for SampleRange {
    fn is_valid(&self) -> bool {
        self.start.is_valid() && self.end.is_valid()
    }
}

impl IsEmpty for SampleRange {
    fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl IsInteger for SampleRange {
    fn is_integer(&self) -> bool {
        self.start.is_integer() && self.end.is_integer()
    }
}
