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

use std::fmt;

///////////////////////////////////////////////////////////////////////
// SampleLayout
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SampleLayout {
    // Samples grouped by channel
    // Example for stereo signal with channels L+R: [LLLL|RRRR]
    Planar,

    // Samples grouped by frame
    // Example for stereo signal with channels L+R: [LR|LR|LR|LR]
    Interleaved,
}

pub type SampleLayoutInvalidity = ();

impl Validate for SampleLayout {
    type Invalidity = SampleLayoutInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        Ok(()) // always valid
    }
}

impl fmt::Display for SampleLayout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

///////////////////////////////////////////////////////////////////////
// SampleFormat
///////////////////////////////////////////////////////////////////////

pub type BitsPerSample = u8;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum SampleFormat {
    Float32,
}

impl fmt::Display for SampleFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", *self)
    }
}

///////////////////////////////////////////////////////////////////////
// SampleType
///////////////////////////////////////////////////////////////////////

pub type SampleType = f32;

///////////////////////////////////////////////////////////////////////
// SamplePosition
///////////////////////////////////////////////////////////////////////

pub type SamplePositionType = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct SamplePosition(pub SamplePositionType);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SamplePositionInvalidity {
    OutOfRange,
}

impl Validate for SamplePosition {
    type Invalidity = SamplePositionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.0.is_finite(), SamplePositionInvalidity::OutOfRange)
            .into()
    }
}

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

impl IsInteger for SamplePosition {
    fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
}

///////////////////////////////////////////////////////////////////////
// SampleLength
///////////////////////////////////////////////////////////////////////

pub type NumberOfSamples = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct SampleLength(pub NumberOfSamples);

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SampleLengthInvalidity {
    OutOfRange,
}

impl Validate for SampleLength {
    type Invalidity = SampleLengthInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(self.0.is_finite() && self.0.is_sign_positive()),
                SampleLengthInvalidity::OutOfRange,
            )
            .into()
    }
}

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

impl IsInteger for SampleLength {
    fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
}
