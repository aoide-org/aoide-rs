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

use std::{f64, fmt};

use crate::prelude::*;

pub type Bpm = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct TempoBpm(Bpm);

impl TempoBpm {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "bpm"
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(f64::MIN_POSITIVE)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(f64::MAX)
    }

    #[must_use]
    pub const fn from_raw(raw: Bpm) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn to_raw(self) -> Bpm {
        let Self(raw) = self;
        raw
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TempoBpmInvalidity {
    OutOfRange,
}

impl Validate for TempoBpm {
    type Invalidity = TempoBpmInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(*self >= Self::min() && *self <= Self::max()),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for TempoBpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}
