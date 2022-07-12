// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
