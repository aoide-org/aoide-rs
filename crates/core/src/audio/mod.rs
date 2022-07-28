// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, time::Duration};

use crate::prelude::*;

pub mod channel;
pub mod sample;
pub mod signal;

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

pub type PositionInMilliseconds = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(transparent)]
pub struct PositionMs(pub PositionInMilliseconds);

impl PositionMs {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "ms"
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
        write!(f, "{:+} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationInMilliseconds = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[repr(transparent)]
pub struct DurationMs(DurationInMilliseconds);

impl DurationMs {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    #[must_use]
    pub const fn from_inner(inner: DurationInMilliseconds) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn to_inner(self) -> DurationInMilliseconds {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self(0_f64)
    }

    #[must_use]
    pub fn is_empty(self) -> bool {
        self <= Self::empty()
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
        let secs = duration.as_secs() as DurationInMilliseconds;
        let subsec_nanos = DurationInMilliseconds::from(duration.subsec_nanos());
        Self(
            secs * DurationInMilliseconds::from(1_000)
                + subsec_nanos / DurationInMilliseconds::from(1_000_000),
        )
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
        write!(f, "{} {}", self.to_inner(), Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
