// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use crate::prelude::*;

pub type TempoBpmValue = f64;

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
pub struct TempoBpm(TempoBpmValue);

impl TempoBpm {
    pub const UNIT_OF_MEASURE: &'static str = "bpm";

    pub const ZERO: Self = Self(0.0);
    pub const MIN: Self = Self(TempoBpmValue::MIN_POSITIVE);
    pub const MAX: Self = Self(TempoBpmValue::MAX);

    #[must_use]
    pub const fn new(value: TempoBpmValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> TempoBpmValue {
        let Self(value) = self;
        value
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
                !(*self >= Self::MIN && *self <= Self::MAX),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for TempoBpm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{value} {unit}",
            value = self.value(),
            unit = Self::UNIT_OF_MEASURE
        )
    }
}
