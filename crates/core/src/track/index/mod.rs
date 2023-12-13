// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

///////////////////////////////////////////////////////////////////////

use std::fmt;

use crate::prelude::*;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Index {
    pub number: Option<u16>,
    pub total: Option<u16>,
}

impl Index {
    pub const MIN_NUMBER: u16 = 1;
    pub const MIN_TOTAL: u16 = 1;
}

#[derive(Copy, Clone, Debug)]
pub enum IndexInvalidity {
    NumberInvalid,
    TotalInvalid,
    NumberExceedsTotal,
}

impl Validate for Index {
    type Invalidity = IndexInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new();
        if let Some(number) = self.number {
            context =
                context.invalidate_if(number < Self::MIN_NUMBER, Self::Invalidity::NumberInvalid);
        }
        if let Some(total) = self.total {
            context =
                context.invalidate_if(total < Self::MIN_TOTAL, Self::Invalidity::TotalInvalid);
        }
        if let (Some(number), Some(total)) = (self.number, self.total) {
            context = context.invalidate_if(number > total, Self::Invalidity::NumberExceedsTotal);
        }
        context.into()
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.number, self.total) {
            (None, None) => f.write_str(""),
            (Some(number), None) => number.fmt(f),
            (None, Some(total)) => write!(f, "/{total}"),
            (Some(number), Some(total)) => write!(f, "{number}/{total}"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Indexes {
    pub disc: Index,
    pub track: Index,
    pub movement: Index,
}

#[derive(Copy, Clone, Debug)]
pub enum IndexesInvalidity {
    Disc(IndexInvalidity),
    Track(IndexInvalidity),
    Movement(IndexInvalidity),
}

impl Validate for Indexes {
    type Invalidity = IndexesInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.disc, Self::Invalidity::Disc)
            .validate_with(&self.track, Self::Invalidity::Track)
            .validate_with(&self.movement, Self::Invalidity::Movement)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
