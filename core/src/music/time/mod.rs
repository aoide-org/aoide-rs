// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{f64, fmt};

///////////////////////////////////////////////////////////////////////
// Tempo
///////////////////////////////////////////////////////////////////////

pub type Beats = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct TempoBpm(pub Beats);

impl TempoBpm {
    pub const fn unit_of_measure() -> &'static str {
        "bpm"
    }

    pub const fn min() -> Self {
        Self(f64::MIN_POSITIVE)
    }

    pub const fn max() -> Self {
        Self(f64::MAX)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TempoBpmInvalidity {
    OutOfRange,
}

impl Validate for TempoBpm {
    type Invalidity = TempoBpmInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::min() && *self <= Self::max()),
                TempoBpmInvalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for TempoBpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

pub type BeatNumber = u16;

/// Musical time signature
///
/// https://en.wikipedia.org/wiki/Time_signature
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeSignature {
    /// The number of beats in each measure unit or bar
    ///
    /// This number appears as the nominator/upper value in the stacked notation.
    pub beats_per_bar: BeatNumber,

    /// The note value that counts as one beat (denominator)
    ///
    /// This number appears as the denominator/lower value in the stacked notation.
    ///
    /// Example: 4 for a quarter-note
    pub beat_unit: Option<BeatNumber>,
}

impl TimeSignature {
    pub fn new(beats_per_bar: BeatNumber, beat_unit: Option<BeatNumber>) -> Self {
        Self { beats_per_bar, beat_unit }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimeSignatureInvalidity {
    Top,
    Bottom,
}

impl Validate for TimeSignature {
    type Invalidity = TimeSignatureInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(self.beats_per_bar < 1, TimeSignatureInvalidity::Top)
            .invalidate_if(
                self.beat_unit.map(|beat_unit| beat_unit < 1).unwrap_or_default(),
                TimeSignatureInvalidity::Bottom,
            )
            .into()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(beat_unit) = self.beat_unit {
            write!(f, "{}/{}", self.beats_per_bar, beat_unit)
        } else {
            write!(f, "{}/", self.beats_per_bar)
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
