// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

///////////////////////////////////////////////////////////////////////

use crate::prelude::*;

use std::fmt;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Index {
    pub number: Option<u16>,
    pub total: Option<u16>,
}

impl Index {
    pub const fn min_number() -> u16 {
        1
    }

    pub const fn min_total() -> u16 {
        1
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
                context.invalidate_if(number < Self::min_number(), Self::Invalidity::NumberInvalid);
        }
        if let Some(total) = self.total {
            context =
                context.invalidate_if(total < Self::min_total(), Self::Invalidity::TotalInvalid);
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
            (Some(number), None) => write!(f, "{}", number),
            (None, Some(total)) => write!(f, "/{}", total),
            (Some(number), Some(total)) => write!(f, "{}/{}", number, total),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Indexes {
    pub disc: Index,
    pub track: Index,
    pub movement: Index,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum IndexesInvalidity {
    Disc(IndexInvalidity),
    Track(IndexInvalidity),
    Movement(IndexInvalidity),
}

impl Validate for Indexes {
    type Invalidity = IndexesInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.disc, IndexesInvalidity::Disc)
            .validate_with(&self.track, IndexesInvalidity::Track)
            .validate_with(&self.movement, IndexesInvalidity::Movement)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
