// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{f64, fmt};

use crate::prelude::*;

pub type Bpm = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
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
    pub const fn new(inner: Bpm) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn to_inner(self) -> Bpm {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
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
