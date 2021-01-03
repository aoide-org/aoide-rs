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

use crate::prelude::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateOrDateTime {
    Date(DateYYYYMMDD),
    DateTime(DateTime),
}

impl From<DateTime> for DateOrDateTime {
    fn from(from: DateTime) -> Self {
        Self::DateTime(from)
    }
}

impl From<DateYYYYMMDD> for DateOrDateTime {
    fn from(from: DateYYYYMMDD) -> Self {
        Self::Date(from)
    }
}

impl From<DateOrDateTime> for DateYYYYMMDD {
    fn from(from: DateOrDateTime) -> Self {
        match from {
            DateOrDateTime::Date(date) => date,
            DateOrDateTime::DateTime(dt) => dt.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DateOrDateTimeInvalidity {
    Date(DateYYYYMMDDInvalidity),
}

impl Validate for DateOrDateTime {
    type Invalidity = DateOrDateTimeInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            DateOrDateTime::Date(date) => context.validate_with(date, Self::Invalidity::Date),
            DateOrDateTime::DateTime(_) => context,
        }
        .into()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Release {
    pub released_at: Option<DateOrDateTime>,

    pub released_by: Option<String>, // record label

    pub copyright: Option<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReleaseInvalidity {
    ReleasedAt(DateOrDateTimeInvalidity),
    ReleasedByEmpty,
    CopyrightEmpty,
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
        context.into()
    }
}
