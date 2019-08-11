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

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
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

impl Validate<()> for TempoBpm {
    fn validate(&self) -> ValidationResult<()> {
        let mut errors = ValidationErrors::default();
        if !(*self >= Self::min() && *self <= Self::max()) {
            errors.add_error((), Violation::OutOfRange);
        }
        errors.into_result()
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

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq)]
pub struct TimeSignature(BeatNumber, BeatNumber);

impl TimeSignature {
    pub fn new(top: BeatNumber, bottom: BeatNumber) -> Self {
        TimeSignature(top, bottom)
    }

    // number of beats in each measure unit or bar, 0 = default/undefined
    pub fn top(self) -> BeatNumber {
        self.0
    }

    pub fn beats_per_measure(self) -> BeatNumber {
        self.top()
    }

    // beat value (the note that counts as one beat), 0 = default/undefined
    pub fn bottom(self) -> BeatNumber {
        self.1
    }

    pub fn measure_unit(self) -> BeatNumber {
        self.bottom()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum TimeSignatureValidation {
    Top,
    Bottom,
}

impl Validate<TimeSignatureValidation> for TimeSignature {
    fn validate(&self) -> ValidationResult<TimeSignatureValidation> {
        let mut errors = ValidationErrors::default();
        if self.top() < 1 {
            errors.add_error(TimeSignatureValidation::Top, Violation::LowerBound);
        }
        if self.bottom() < 1 {
            errors.add_error(TimeSignatureValidation::Bottom, Violation::LowerBound);
        }
        errors.into_result()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}/{}", self.top(), self.bottom())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
