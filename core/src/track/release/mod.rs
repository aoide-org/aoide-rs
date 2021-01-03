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

use super::*;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Release {
    pub released_at: Option<DateOrDateTime>,

    pub released_by: Option<String>, // record label

    pub copyright: Option<String>,

    pub licenses: Vec<String>,
}

impl Release {
    pub fn date(&self) -> Option<Date> {
        self.released_at.map(Into::into)
    }

    pub fn year(&self) -> Option<YearType> {
        self.date().map(Date::year)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReleaseInvalidity {
    ReleasedAt(DateOrDateTimeInvalidity),
    ReleasedByEmpty,
    CopyrightEmpty,
    LicenseEmpty,
}

impl Validate for Release {
    type Invalidity = ReleaseInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new()
            .validate_with(&self.released_at, ReleaseInvalidity::ReleasedAt);
        if let Some(ref released_by) = self.released_by {
            context = context.invalidate_if(
                released_by.trim().is_empty(),
                ReleaseInvalidity::ReleasedByEmpty,
            );
        }
        if let Some(ref copyright) = self.copyright {
            context = context.invalidate_if(
                copyright.trim().is_empty(),
                ReleaseInvalidity::CopyrightEmpty,
            );
        }
        self.licenses
            .iter()
            .fold(context, |context, license| {
                context.invalidate_if(license.trim().is_empty(), ReleaseInvalidity::LicenseEmpty)
            })
            .into()
    }
}
