// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use chrono::{
    DateTime, Datelike, FixedOffset, NaiveDate, NaiveDateTime, ParseError, SecondsFormat,
};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReleaseDateTime(DateTime<FixedOffset>);

impl From<DateTime<FixedOffset>> for ReleaseDateTime {
    fn from(from: DateTime<FixedOffset>) -> Self {
        Self(from)
    }
}

impl From<ReleaseDateTime> for DateTime<FixedOffset> {
    fn from(from: ReleaseDateTime) -> Self {
        from.0
    }
}

impl FromStr for ReleaseDateTime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl ToString for ReleaseDateTime {
    fn to_string(&self) -> String {
        self.0.to_rfc3339_opts(SecondsFormat::Secs, true)
    }
}

// 4-digit year
pub type ReleaseYear = i16;

// 2-digit month
pub type ReleaseMonth = i8;

// 2-digit day of month
pub type ReleaseDayOfMonth = i8;

pub const RELEASE_YEAR_MIN: ReleaseYear = 1;
pub const RELEASE_YEAR_MAX: ReleaseYear = 9999;

pub type YYYYMMDD = i32;

// 8-digit year+month+day (YYYYMMDD)
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReleaseDate(YYYYMMDD);

impl ReleaseDate {
    pub const fn min() -> Self {
        Self(10_000)
    }

    pub const fn max() -> Self {
        Self(99_999_999)
    }

    pub const fn new(val: YYYYMMDD) -> Self {
        Self(val)
    }

    pub fn year(self) -> ReleaseYear {
        (self.0 / 10_000) as ReleaseYear
    }

    pub fn month(self) -> ReleaseMonth {
        ((self.0 % 10_000) / 100) as ReleaseMonth
    }

    pub fn day_of_month(self) -> ReleaseDayOfMonth {
        (self.0 % 100) as ReleaseDayOfMonth
    }

    pub fn from_year(year: ReleaseYear) -> Self {
        Self(YYYYMMDD::from(year) * 10_000)
    }

    pub fn is_year(self) -> bool {
        Self::from_year(self.year()) == self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReleaseDateInvalidity {
    Min,
    Max,
    MonthOutOfRange,
    DayOfMonthOutOfRange,
    DayWithoutMonth,
    Invalid,
}

impl Validate for ReleaseDate {
    type Invalidity = ReleaseDateInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::min(), ReleaseDateInvalidity::Min)
            .invalidate_if(*self > Self::max(), ReleaseDateInvalidity::Min)
            .invalidate_if(
                self.month() < 0 || self.month() > 12,
                ReleaseDateInvalidity::MonthOutOfRange,
            )
            .invalidate_if(
                self.day_of_month() < 0 || self.day_of_month() > 31,
                ReleaseDateInvalidity::DayOfMonthOutOfRange,
            )
            .invalidate_if(
                self.month() < 1 && self.day_of_month() > 0,
                ReleaseDateInvalidity::DayWithoutMonth,
            )
            .invalidate_if(
                self.month() > 0
                    && self.day_of_month() > 0
                    && NaiveDate::from_ymd_opt(
                        i32::from(self.year()),
                        self.month() as u32,
                        self.day_of_month() as u32,
                    )
                    .is_none(),
                ReleaseDateInvalidity::Invalid,
            )
            .into()
    }
}

impl From<NaiveDateTime> for ReleaseDate {
    fn from(from: NaiveDateTime) -> Self {
        Self(
            from.year() as YYYYMMDD * 10_000
                + from.month() as YYYYMMDD * 100
                + from.day() as YYYYMMDD,
        )
    }
}

impl From<ReleaseDateTime> for ReleaseDate {
    fn from(from: ReleaseDateTime) -> Self {
        from.0.naive_local().into()
    }
}

impl From<ReleaseDate> for YYYYMMDD {
    fn from(from: ReleaseDate) -> Self {
        from.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReleasedAt {
    Date(ReleaseDate),
    DateTime(ReleaseDateTime),
}

impl From<ReleasedAt> for ReleaseDate {
    fn from(from: ReleasedAt) -> Self {
        match from {
            ReleasedAt::Date(date) => date,
            ReleasedAt::DateTime(dt) => dt.into(),
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReleasedAtInvalidity {
    Date(ReleaseDateInvalidity),
}

impl Validate for ReleasedAt {
    type Invalidity = ReleasedAtInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let context = ValidationContext::new();
        match self {
            ReleasedAt::Date(date) => context.validate_with(date, ReleasedAtInvalidity::Date),
            ReleasedAt::DateTime(_) => context,
        }
        .into()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Release {
    pub released_at: Option<ReleasedAt>,

    pub released_by: Option<String>, // record label

    pub copyright: Option<String>,

    pub licenses: Vec<String>,
}

impl Release {
    pub fn date(&self) -> Option<ReleaseDate> {
        self.released_at.map(Into::into)
    }

    pub fn year(&self) -> Option<ReleaseYear> {
        self.date().map(ReleaseDate::year)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReleaseInvalidity {
    ReleasedAt(ReleasedAtInvalidity),
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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
