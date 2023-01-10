// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, ops::Deref, str::FromStr, time::SystemTime};

use time::{
    error::{IndeterminateOffset, Parse as ParseError},
    format_description::{well_known::Rfc3339, FormatItem},
    Date, Duration, Month, OffsetDateTime,
};

use crate::prelude::*;

pub type DateTimeInner = OffsetDateTime;

pub type TimestampMillis = i64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct DateTime(DateTimeInner);

const NANOS_PER_MILLISECOND: i128 = 1_000_000;

const YYYY_MM_DD_FORMAT: &[FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");

/// A `DateTime` with truncated millisecond precision.
impl DateTime {
    #[must_use]
    pub fn new(inner: DateTimeInner) -> Self {
        let subsec_nanos_since_last_millis_boundary =
            inner.unix_timestamp_nanos() % NANOS_PER_MILLISECOND;
        let subsec_duration_since_last_millis_boundary =
            Duration::nanoseconds(subsec_nanos_since_last_millis_boundary as i64);
        let truncated = inner - subsec_duration_since_last_millis_boundary;
        debug_assert_eq!(0, truncated.unix_timestamp_nanos() % NANOS_PER_MILLISECOND);
        Self(truncated)
    }

    #[must_use]
    pub fn new_timestamp_millis(timestamp_millis: TimestampMillis) -> Self {
        DateTimeInner::from_unix_timestamp_nanos(
            i128::from(timestamp_millis) * NANOS_PER_MILLISECOND,
        )
        .expect("valid timestamp")
        .into()
    }

    #[must_use]
    pub const fn to_inner(self) -> DateTimeInner {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn now_utc() -> Self {
        DateTimeInner::now_utc().into()
    }

    #[must_use]
    pub fn now_local_or_utc() -> Self {
        DateTimeInner::now_local().map_or_else(|_: IndeterminateOffset| Self::now_utc(), Into::into)
    }

    #[must_use]
    pub fn timestamp_millis(self) -> TimestampMillis {
        (self.to_inner().unix_timestamp_nanos() / NANOS_PER_MILLISECOND) as TimestampMillis
    }

    #[must_use]
    pub fn year(&self) -> YearType {
        self.0.year() as _
    }
}

impl AsRef<DateTimeInner> for DateTime {
    fn as_ref(&self) -> &DateTimeInner {
        &self.0
    }
}

impl Deref for DateTime {
    type Target = DateTimeInner;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
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

impl From<SystemTime> for DateTime {
    fn from(from: SystemTime) -> Self {
        Self::new(from.into())
    }
}

impl From<DateTime> for SystemTime {
    fn from(from: DateTime) -> Self {
        from.to_inner().into()
    }
}

impl FromStr for DateTime {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        DateTimeInner::parse(input, &Rfc3339).map(Into::into)
    }
}

impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Avoid allocation of temporary String?
        f.write_str(&self.to_inner().format(&Rfc3339).expect("valid timestamp"))
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(clippy::upper_case_acronyms)]
#[repr(transparent)]
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

#[derive(Copy, Clone, Debug)]
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
                self.month() >= 1
                    && self.month() <= 12
                    && self.day_of_month() >= 1
                    && self.day_of_month() <= 31
                    && Date::from_calendar_date(
                        self.year().into(),
                        Month::try_from(self.month() as u8).expect("valid month"),
                        self.day_of_month() as u8,
                    )
                    .is_err(),
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
        from.date().into()
    }
}

impl From<Date> for DateYYYYMMDD {
    #[allow(clippy::cast_possible_wrap)]
    fn from(from: Date) -> Self {
        Self(
            from.year() as YYYYMMDD * 10_000
                + from.month() as YYYYMMDD * 100
                + YYYYMMDD::from(from.day()),
        )
    }
}

impl fmt::Display for DateYYYYMMDD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_year() {
            return write!(f, "{:04}", self.year());
        }
        if self.month() >= 1 && self.month() <= 12 && self.day_of_month() <= 31 {
            if let Ok(date) = Date::from_calendar_date(
                self.year().into(),
                Month::try_from(self.month() as u8).expect("valid month"),
                self.day_of_month() as u8,
            ) {
                // TODO: Avoid allocation of temporary String?
                return f.write_str(&date.format(YYYY_MM_DD_FORMAT).expect("valid date"));
            }
        }
        if self.day_of_month() == 0 {
            return write!(f, "{:04}-{:02}", self.year(), self.month());
        }
        // Fallback
        let Self(inner) = self;
        write!(f, "{inner:08}")
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DateOrDateTime {
    Date(DateYYYYMMDD),
    DateTime(DateTime),
}

impl DateOrDateTime {
    #[must_use]
    pub fn year(self) -> YearType {
        match self {
            Self::Date(inner) => inner.year(),
            Self::DateTime(inner) => inner.year(),
        }
    }
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
            Self::Date(date) => write!(f, "{date}"),
            Self::DateTime(datetime) => write!(f, "{datetime}"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
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
