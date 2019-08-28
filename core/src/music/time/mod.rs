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

use std::{f64, fmt};

///////////////////////////////////////////////////////////////////////
// Tempo
///////////////////////////////////////////////////////////////////////

pub type Beats = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

pub type BeatNumber = u16;

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq)]
pub struct TimeSignature {
    // number of beats in each measure unit or bar, 0 = default/undefined
    pub top: BeatNumber,

    // beat value (the note that counts as one beat), 0 = default/undefined
    pub bottom: BeatNumber,
}

impl TimeSignature {
    pub fn new(top: BeatNumber, bottom: BeatNumber) -> Self {
        Self { top, bottom }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimeSignatureInvalidity {
    TopLowerBound(BeatNumber, BeatNumber),
    BottomLowerBound(BeatNumber, BeatNumber),
}

impl Validate for TimeSignature {
    type Invalidity = TimeSignatureInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.top < 1,
                TimeSignatureInvalidity::TopLowerBound(1, self.top),
            )
            .invalidate_if(
                self.bottom < 1,
                TimeSignatureInvalidity::BottomLowerBound(1, self.bottom),
            )
            .into()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.top, self.bottom)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
