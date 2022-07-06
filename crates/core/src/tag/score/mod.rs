// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::prelude::*;

pub type ScoreValue = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Score(ScoreValue);

impl Score {
    #[must_use]
    pub const fn min_value() -> ScoreValue {
        0.0
    }

    #[must_use]
    pub const fn max_value() -> ScoreValue {
        1.0
    }

    #[must_use]
    pub const fn default_value() -> ScoreValue {
        Self::max_value()
    }

    pub fn clamp_value(value: impl Into<ScoreValue>) -> ScoreValue {
        value.into().clamp(Self::min_value(), Self::max_value())
    }

    pub fn clamp_from(value: impl Into<ScoreValue>) -> Self {
        Self::clamp_value(value).into()
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(Self::min_value())
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(Self::max_value())
    }

    #[must_use]
    pub const fn default() -> Self {
        Self(Self::default_value())
    }

    #[must_use]
    pub const fn new(inner: ScoreValue) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn value(self) -> ScoreValue {
        let Self(value) = self;
        value
    }

    // Convert to percentage value with a single decimal digit
    #[must_use]
    pub fn to_percentage(self) -> ScoreValue {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() / ScoreValue::from(10)
    }

    // Convert to an integer permille value
    #[must_use]
    pub fn to_permille(self) -> u16 {
        debug_assert!(self.validate().is_ok());
        (self.value() * ScoreValue::from(1_000)).round() as u16
    }
}

impl Default for Score {
    fn default() -> Self {
        Self::default()
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
                !(*self >= Self::min() && *self <= Self::max()),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl From<Score> for ScoreValue {
    fn from(from: Score) -> Self {
        from.value()
    }
}

impl From<ScoreValue> for Score {
    fn from(value: ScoreValue) -> Self {
        Self::new(value)
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        debug_assert!(self.validate().is_ok());
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