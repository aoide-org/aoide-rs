// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, time::Duration};

use semval::prelude::*;

pub mod channel;
pub use self::channel::*;

pub mod sample;
pub use self::sample::*;

pub mod signal;
pub use self::signal::*;

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

pub type PositionMsValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
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
pub struct PositionMs(PositionMsValue);

impl PositionMs {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    #[must_use]
    pub const fn new(value: DurationMsValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> DurationMsValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PositionMsInvalidity {
    OutOfRange,
}

impl Validate for PositionMs {
    type Invalidity = PositionMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.0.is_finite(), Self::Invalidity::OutOfRange)
            .into()
    }
}

impl fmt::Display for PositionMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{value:+} {unit}",
            value = self.value(),
            unit = Self::UNIT_OF_MEASURE
        )
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationMsValue = f64;

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
pub struct DurationMs(DurationMsValue);

impl DurationMs {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    #[must_use]
    pub const fn new(value: DurationMsValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> DurationMsValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self(0_f64)
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self <= Self::empty()
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DurationMsInvalidity {
    OutOfRange,
}

impl Validate for DurationMs {
    type Invalidity = DurationMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(self.0.is_finite() && *self >= Self::empty()),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl From<Duration> for DurationMs {
    fn from(duration: Duration) -> Self {
        let secs = duration.as_secs() as DurationMsValue;
        let subsec_nanos = DurationMsValue::from(duration.subsec_nanos());
        Self(secs * DurationMsValue::from(1_000) + subsec_nanos / DurationMsValue::from(1_000_000))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DurationOutOfRangeError;

impl TryFrom<DurationMs> for Duration {
    type Error = DurationOutOfRangeError;

    fn try_from(value: DurationMs) -> Result<Self, Self::Error> {
        let millis = value.0;
        if !millis.is_finite() || millis < 0.0 {
            return Err(DurationOutOfRangeError);
        }
        let secs = millis / 1_000.0;
        if secs > Duration::MAX.as_secs_f64() {
            return Err(DurationOutOfRangeError);
        }
        Ok(Self::from_secs_f64(millis / 1_000.0))
    }
}

impl fmt::Display for DurationMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{value} {unit}",
            value = self.value(),
            unit = Self::UNIT_OF_MEASURE
        )
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
