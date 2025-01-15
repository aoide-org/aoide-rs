// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use semval::prelude::*;

use crate::util::IsInteger;

///////////////////////////////////////////////////////////////////////
// SampleLayout
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Display)]
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

///////////////////////////////////////////////////////////////////////
// SampleFormat
///////////////////////////////////////////////////////////////////////

pub type BitsPerSample = u8;

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::Display)]
pub enum SampleFormat {
    Float32,
}

///////////////////////////////////////////////////////////////////////
// SampleType
///////////////////////////////////////////////////////////////////////

pub type SampleType = f32;

///////////////////////////////////////////////////////////////////////
// SamplePosition
///////////////////////////////////////////////////////////////////////

pub type SamplePositionValue = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, derive_more::Display)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct SamplePosition(SamplePositionValue);

impl SamplePosition {
    #[must_use]
    pub const fn new(value: SamplePositionValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> SamplePositionValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

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

impl IsInteger for SamplePosition {
    fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
}

///////////////////////////////////////////////////////////////////////
// SampleLength
///////////////////////////////////////////////////////////////////////

pub type SampleLengthValue = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
pub struct SampleLength(SampleLengthValue);

impl SampleLength {
    #[must_use]
    pub const fn new(value: SampleLengthValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> SampleLengthValue {
        let Self(value) = self;
        value
    }

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
                !(self.value().is_finite() && self.value().is_sign_positive()),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl IsInteger for SampleLength {
    fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
}
