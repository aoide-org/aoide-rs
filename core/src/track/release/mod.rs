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

use chrono::{DateTime, Datelike, Utc};

///////////////////////////////////////////////////////////////////////
// ReleaseMetadata
///////////////////////////////////////////////////////////////////////

pub type ReleaseYear = i16;

pub const MIN_RELEASE_YEAR: ReleaseYear = 0;

pub const MAX_RELEASE_YEAR: ReleaseYear = std::i16::MAX;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Release {
    pub released_at: Option<DateTime<Utc>>,

    pub released_by: Option<String>, // record label

    pub copyright: Option<String>,

    pub licenses: Vec<String>,
}

impl Release {
    pub fn release_year(&self) -> Option<ReleaseYear> {
        self.released_at
            .map(|released_at| released_at.year() as ReleaseYear)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReleaseValidation {
    ReleasedAt,
    ReleasedBy,
    Copyright,
    License,
}

const RELEASED_BY_MIN_LEN: usize = 1;

const COPYRIGHT_MIN_LEN: usize = 1;

const LICENSE_MIN_LEN: usize = 1;

impl Validate<ReleaseValidation> for Release {
    fn validate(&self) -> ValidationResult<ReleaseValidation> {
        let mut errors = ValidationErrors::default();
        if let Some(released_at) = self.released_at {
            let year = released_at.year();
            if year < i32::from(MIN_RELEASE_YEAR) || year > i32::from(MAX_RELEASE_YEAR) {
                errors.add_error(ReleaseValidation::ReleasedAt, Violation::OutOfRange);
            }
        }
        if let Some(ref released_by) = self.released_by {
            if released_by.len() < RELEASED_BY_MIN_LEN {
                errors.add_error(
                    ReleaseValidation::ReleasedBy,
                    Violation::too_short(RELEASED_BY_MIN_LEN),
                );
            }
        }
        if let Some(ref copyright) = self.copyright {
            if copyright.len() < COPYRIGHT_MIN_LEN {
                errors.add_error(
                    ReleaseValidation::Copyright,
                    Violation::too_short(COPYRIGHT_MIN_LEN),
                );
            }
        }
        for license in &self.licenses {
            if license.len() < LICENSE_MIN_LEN {
                errors.add_error(
                    ReleaseValidation::License,
                    Violation::too_short(LICENSE_MIN_LEN),
                );
                break;
            }
        }
        errors.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

// TODO
