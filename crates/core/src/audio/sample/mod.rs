// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

use std::fmt;

///////////////////////////////////////////////////////////////////////
// SampleLayout
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(transparent)]
pub struct SamplePosition(pub SamplePositionType);

#[derive(Copy, Clone, Debug)]
pub enum SamplePositionInvalidity {
    OutOfRange,
}

impl Validate for SamplePosition {
    type Invalidity = SamplePositionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.0.is_finite(), Self::Invalidity::OutOfRange)
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[repr(transparent)]
pub struct SampleLength(pub NumberOfSamples);

impl SampleLength {
    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SampleLengthInvalidity {
    OutOfRange,
}

impl Validate for SampleLength {
    type Invalidity = SampleLengthInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(self.0.is_finite() && self.0.is_sign_positive()),
                Self::Invalidity::OutOfRange,
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
