// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use chrono::{
    Datelike, Duration, FixedOffset, Local, NaiveDate, NaiveDateTime, ParseError, SecondsFormat,
    TimeZone, Utc,
};
use std::{fmt, str::FromStr, time::SystemTime};

pub type DateTimeInner = chrono::DateTime<FixedOffset>;

pub type TimestampMillis = i64;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct DateTime(DateTimeInner);

const NANOS_PER_MILLISECOND: u32 = 1_000_000;

/// A DateTime with truncated millisecond precision.
impl DateTime {
    #[must_use]
    pub fn new(inner: DateTimeInner) -> Self {
        let subsec_duration_since_last_millis_boundary =
            Duration::nanoseconds((inner.timestamp_subsec_nanos() % NANOS_PER_MILLISECOND).into());
        let truncated = inner - subsec_duration_since_last_millis_boundary;
        debug_assert_eq!(
            0,
            truncated.timestamp_subsec_nanos() % NANOS_PER_MILLISECOND
        );
        Self(truncated)
    }

    #[must_use]
    pub fn new_timestamp_millis(timestamp_millis: TimestampMillis) -> Self {
        Utc.timestamp_millis(timestamp_millis).into()
    }

    #[must_use]
    pub const fn to_inner(self) -> DateTimeInner {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn now_utc() -> Self {
        Utc::now().into()
    }

    #[must_use]
    pub fn now_local() -> Self {
        Local::now().into()
    }

    #[must_use]
    pub fn naive_date(self) -> NaiveDate {
        self.to_inner().naive_local().date()
    }

    #[must_use]
    pub fn timestamp_millis(self) -> TimestampMillis {
        self.to_inner().timestamp_millis()
    }
}

impl AsRef<DateTimeInner> for DateTime {
    fn as_ref(&self) -> &DateTimeInner {
        &self.0
    }
}

impl From<DateTimeInner> for DateTime {
    fn from(from: DateTimeInner) -> Self {
        Self::new(from)
    }
}

impl From<DateTime> for DateTimeInner {
    fn from(from: DateTime) -> Self {
        from.to_inner()
    }
}

impl From<chrono::DateTime<Utc>> for DateTime {
    fn from(from: chrono::DateTime<Utc>) -> Self {
        Self::new(from.into())
    }
}

impl From<DateTime> for chrono::DateTime<Utc> {
    fn from(from: DateTime) -> Self {
        from.to_inner().into()
    }
}

impl From<chrono::DateTime<Local>> for DateTime {
    fn from(from: chrono::DateTime<Local>) -> Self {
        Self::new(from.into())
    }
}

impl From<DateTime> for chrono::DateTime<Local> {
    fn from(from: DateTime) -> Self {
        from.to_inner().into()
    }
}

impl From<SystemTime> for DateTime {
    fn from(from: SystemTime) -> Self {
        chrono::DateTime::<Utc>::from(from).into()
    }
}

impl FromStr for DateTime {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s.parse()?))
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.to_inner().to_rfc3339_opts(SecondsFormat::AutoSi, true)
        )
    }
}

// 4-digit year
pub type YearType = i16;

// 2-digit month
pub type MonthType = i8;

// 2-digit day of month
pub type DayOfMonthType = i8;

pub const YEAR_MIN: YearType = 1;
pub const YEAR_MAX: YearType = 9999;

// 8-digit year+month+day (YYYYMMDD)
#[allow(clippy::upper_case_acronyms)]
pub type YYYYMMDD = i32;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[allow(clippy::upper_case_acronyms)]
pub struct DateYYYYMMDD(YYYYMMDD);

impl DateYYYYMMDD {
    #[must_use]
    pub const fn min() -> Self {
        Self(10_000)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(99_991_231)
    }

    #[must_use]
    pub const fn new(val: YYYYMMDD) -> Self {
        Self(val)
    }

    #[must_use]
    pub const fn to_inner(self) -> YYYYMMDD {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn year(self) -> YearType {
        (self.0 / 10_000) as YearType
    }

    #[must_use]
    pub fn month(self) -> MonthType {
        ((self.0 % 10_000) / 100) as MonthType
    }

    #[must_use]
    pub fn day_of_month(self) -> DayOfMonthType {
        (self.0 % 100) as DayOfMonthType
    }

    #[must_use]
    pub fn from_year(year: YearType) -> Self {
        Self(YYYYMMDD::from(year) * 10_000)
    }

    #[must_use]
    pub fn from_year_month(year: YearType, month: MonthType) -> Self {
        Self(YYYYMMDD::from(year) * 10_000 + YYYYMMDD::from(month) * 100)
    }

    #[must_use]
    pub fn is_year(self) -> bool {
        Self::from_year(self.year()) == self
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum DateYYYYMMDDInvalidity {
    Min,
    Max,
    MonthOutOfRange,
    DayOfMonthOutOfRange,
    DayWithoutMonth,
    Invalid,
}

impl Validate for DateYYYYMMDD {
    type Invalidity = DateYYYYMMDDInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::min(), Self::Invalidity::Min)
            .invalidate_if(*self > Self::max(), Self::Invalidity::Min)
            .invalidate_if(
                self.month() < 0 || self.month() > 12,
                Self::Invalidity::MonthOutOfRange,
            )
            .invalidate_if(
                self.day_of_month() < 0 || self.day_of_month() > 31,
                Self::Invalidity::DayOfMonthOutOfRange,
            )
            .invalidate_if(
                self.month() < 1 && self.day_of_month() > 0,
                Self::Invalidity::DayWithoutMonth,
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
                Self::Invalidity::Invalid,
            )
            .into()
    }
}

impl From<YYYYMMDD> for DateYYYYMMDD {
    fn from(from: YYYYMMDD) -> Self {
        Self::new(from)
    }
}

impl From<DateYYYYMMDD> for YYYYMMDD {
    fn from(from: DateYYYYMMDD) -> Self {
        from.to_inner()
    }
}

impl From<DateTime> for DateYYYYMMDD {
    fn from(from: DateTime) -> Self {
        from.naive_date().into()
    }
}

impl From<NaiveDate> for DateYYYYMMDD {
    #[allow(clippy::cast_possible_wrap)]
    fn from(from: NaiveDate) -> Self {
        Self(
            from.year() as YYYYMMDD * 10_000
                + from.month() as YYYYMMDD * 100
                + from.day() as YYYYMMDD,
        )
    }
}

impl From<NaiveDateTime> for DateYYYYMMDD {
    fn from(from: NaiveDateTime) -> Self {
        from.date().into()
    }
}

impl fmt::Display for DateYYYYMMDD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_year() {
            return write!(f, "{:04}", self.year());
        }
        if let Some(date) = NaiveDate::from_ymd_opt(
            self.year().into(),
            self.month() as u32,
            self.day_of_month() as u32,
        ) {
            return write!(f, "{}", date.format("%Y-%m-%d"));
        }
        if self.day_of_month() == 0 {
            return write!(f, "{:04}-{:02}", self.year(), self.month());
        }
        // Fallback
        let Self(inner) = self;
        write!(f, "{:08}", inner)
    }
}

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

impl PartialOrd for DateOrDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Self::Date(lhs), Self::Date(rhs)) => lhs.partial_cmp(rhs),
            (Self::DateTime(lhs), Self::DateTime(rhs)) => lhs.partial_cmp(rhs),
            (Self::Date(_), Self::DateTime(_)) | (Self::DateTime(_), Self::Date(_)) => None,
        }
    }
}

impl fmt::Display for DateOrDateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date(date) => write!(f, "{}", date),
            Self::DateTime(datetime) => write!(f, "{}", datetime),
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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
