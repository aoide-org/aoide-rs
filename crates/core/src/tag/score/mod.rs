// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use crate::prelude::*;

pub type ScoreValue = f64;

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
pub struct Score(ScoreValue);

impl Score {
    pub const MIN_VALUE: ScoreValue = 0.0;

    pub const MAX_VALUE: ScoreValue = 1.0;

    pub const DEFAULT_VALUE: ScoreValue = Self::MAX_VALUE;

    pub const MIN: Self = Self(Self::MIN_VALUE);

    pub const MAX: Self = Self(Self::MAX_VALUE);

    pub const DEFAULT: Self = Self(Self::DEFAULT_VALUE);

    pub fn clamp_value(value: impl Into<ScoreValue>) -> ScoreValue {
        value.into().clamp(Self::MIN_VALUE, Self::MAX_VALUE)
    }

    pub fn clamp_from(value: impl Into<ScoreValue>) -> Self {
        Self::new_unchecked(Self::clamp_value(value))
    }

    #[must_use]
    pub const fn new_unchecked(value: ScoreValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> ScoreValue {
        let Self(value) = self;
        value
    }

    // Convert to percentage value with a single decimal digit
    #[must_use]
    pub fn to_percentage(self) -> ScoreValue {
        debug_assert!(self.is_valid());
        (self.value() * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
    #[must_use]
    pub fn to_permille(self) -> u16 {
        debug_assert!(self.is_valid());
        (self.value() * ScoreValue::from(1_000)).round() as u16
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[derive(Copy, Clone, Debug)]
pub enum ScoreInvalidity {
    OutOfRange,
}

impl Validate for Score {
    type Invalidity = ScoreInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::MIN && *self <= Self::MAX),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_assert!(self.is_valid());
        write!(f, "{:.1}%", self.to_percentage())
    }
}

pub trait Scored {
    fn score(&self) -> Score;
}

impl Scored for Score {
    fn score(&self) -> Self {
        *self
    }
}

#[cfg(test)]
mod tests;
