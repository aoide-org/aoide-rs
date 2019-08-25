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

///////////////////////////////////////////////////////////////////////

use super::*;

use std::fmt;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Index {
    pub number: u16,
    pub total: u16,
}

impl Index {
    pub const fn min_number() -> u16 {
        1
    }

    pub const fn min_total() -> u16 {
        1
    }

    pub fn number(self) -> Option<u16> {
        if self.number < Self::min_number() {
            None
        } else {
            Some(self.number)
        }
    }

    pub fn total(self) -> Option<u16> {
        if self.total < Self::min_total() {
            None
        } else {
            Some(self.total)
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IndexValidation {
    SingleExceedsTotal,
}

impl Validate for Index {
    type Validation = IndexValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        if let (Some(number), Some(total)) = (self.number(), self.total()) {
            if number > total {
                context.add_violation(IndexValidation::SingleExceedsTotal);
            }
        }
        context.into_result()
    }
}

impl fmt::Display for Index {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match (self.number(), self.total()) {
            (None, None) => f.write_str(""),
            (Some(number), None) => write!(f, "{}", number),
            (None, Some(total)) => write!(f, "/{}", total),
            (Some(number), Some(total)) => write!(f, "{}/{}", number, total),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Indexes {
    pub disc: Index,
    pub track: Index,
    pub movement: Index,
}

#[derive(Copy, Clone, Debug)]
pub enum IndexesValidation {
    Disc(IndexValidation),
    Track(IndexValidation),
    Movement(IndexValidation),
}

impl Validate for Indexes {
    type Validation = IndexesValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.disc.validate(), IndexesValidation::Disc);
        context.map_and_merge_result(self.track.validate(), IndexesValidation::Track);
        context.map_and_merge_result(self.movement.validate(), IndexesValidation::Movement);
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(tests)]
mod tests;
