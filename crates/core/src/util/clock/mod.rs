// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, str::FromStr};

use jiff::Zoned;
use semval::prelude::*;
use time::{
    convert::{Millisecond, Nanosecond, Second},
    error::Parse as ParseError,
    format_description::{well_known::Rfc3339, FormatItem},
    Date, Duration, Month, OffsetDateTime, UtcOffset,
};

pub type TimestampMillis = i64;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct OffsetDateTimeMs(OffsetDateTime);

// Serialize (and deserialize) as string for maximum compatibility and portability
#[cfg(feature = "serde")]
impl serde::Serialize for OffsetDateTimeMs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        time::serde::rfc3339::serialize(&self.0, serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for OffsetDateTimeMs {
    fn deserialize<D>(deserializer: D) -> Result<OffsetDateTimeMs, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        time::serde::rfc3339::deserialize(deserializer).map(OffsetDateTimeMs::clamp_from)
    }
}

const YYYY_MM_DD_FORMAT: &[FormatItem<'static>] =
    time::macros::format_description!("[year]-[month]-[day]");

/// An [`OffsetDateTime`] with truncated millisecond precision.
impl OffsetDateTimeMs {
    #[must_use]
    pub const fn new_unchecked(inner: OffsetDateTime) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn clamp_from(inner: OffsetDateTime) -> Self {
        // Avoid i128 arithmetic operations to maximize performance.
        let subsec_nanos_mod = inner.nanosecond() % Nanosecond::per(Millisecond);
        let subsec_duration_mod = Duration::nanoseconds(subsec_nanos_mod.into());
        let truncated = Self::new_unchecked(inner - subsec_duration_mod);
        debug_assert!(truncated.is_valid());
        truncated
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Never panics
    pub fn from_timestamp_millis(timestamp_millis: TimestampMillis) -> Self {
        // Avoid i128 arithmetic operations to maximize performance.
        // TODO: Use https://doc.rust-lang.org/std/primitive.i64.html#method.div_floor when available.
        let seconds: i64 =
            num_integer::div_floor(timestamp_millis, i64::from(Millisecond::per(Second)));
        let milliseconds_floor = seconds * i64::from(Millisecond::per(Second));
        debug_assert!(milliseconds_floor <= timestamp_millis);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let subsec_nanos =
            (timestamp_millis - milliseconds_floor) as u32 * Nanosecond::per(Millisecond);
        let truncated = Self::clamp_from(
            OffsetDateTime::from_unix_timestamp(seconds).expect("valid timestamp")
                + Duration::nanoseconds(subsec_nanos.into()),
        );
        debug_assert!(truncated.is_valid());
        truncated
    }

    #[must_use]
    pub fn now_utc() -> Self {
        Self::clamp_from(OffsetDateTime::now_utc())
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)] // Never panics.
    pub fn now_local() -> Self {
        let zoned = Zoned::now();
        let ts_millis = zoned.timestamp().as_millisecond();
        let this_utc = Self::from_timestamp_millis(ts_millis);
        let offset_secs = zoned.offset().seconds();
        let utc_offset =
            UtcOffset::from_whole_seconds(offset_secs).expect("offset should always be valid");
        Self(this_utc.0.replace_offset(utc_offset))
    }

    #[must_use]
    pub fn timestamp_millis(&self) -> TimestampMillis {
        // Avoid i128 arithmetic operations to maximize performance.
        let seconds = self.0.unix_timestamp();
        let subsec_millis = self.0.nanosecond() / Nanosecond::per(Millisecond);
        seconds * TimestampMillis::from(Millisecond::per(Second))
            + TimestampMillis::from(subsec_millis)
    }

    #[must_use]
    pub const fn year(&self) -> YearType {
        self.0.year() as _
    }

    #[must_use]
    pub const fn date(&self) -> Date {
        self.0.date()
    }

    #[must_use]
    pub const fn date_time(&self) -> OffsetDateTime {
        self.0
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

impl FromStr for OffsetDateTimeMs {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        OffsetDateTime::parse(input, &Rfc3339).map(Self::clamp_from)
    }
}

impl fmt::Display for OffsetDateTimeMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: Avoid allocation of temporary String?
        self.0.format(&Rfc3339).expect("valid timestamp").fmt(f)
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum OffsetDateTimeMsInvalidity {
    /// Higher precision than expected
    Unclamped,
}

impl Validate for OffsetDateTimeMs {
    type Invalidity = OffsetDateTimeMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.0.nanosecond() % Nanosecond::per(Millisecond) != 0,
                Self::Invalidity::Unclamped,
            )
            .into()
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

pub type YyyyMmDdDateValue = i32;

/// 8-digit year+month+day (YYYYMMDD)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct YyyyMmDdDate(YyyyMmDdDateValue);

impl YyyyMmDdDate {
    pub const MIN: Self = Self(10_000);

    pub const MAX: Self = Self(99_991_231);

    #[must_use]
    pub const fn new_unchecked(value: YyyyMmDdDateValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> YyyyMmDdDateValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub const fn year(self) -> YearType {
        (self.0 / 10_000) as YearType
    }

    #[must_use]
    pub const fn month(self) -> MonthType {
        ((self.0 % 10_000) / 100) as MonthType
    }

    #[must_use]
    pub const fn day_of_month(self) -> DayOfMonthType {
        (self.0 % 100) as DayOfMonthType
    }

    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn from_date(from: Date) -> Self {
        Self(
            from.year() as YyyyMmDdDateValue * 10_000
                + from.month() as YyyyMmDdDateValue * 100
                + YyyyMmDdDateValue::from(from.day()),
        )
    }

    #[must_use]
    pub fn from_year(year: YearType) -> Self {
        Self(YyyyMmDdDateValue::from(year) * 10_000)
    }

    #[must_use]
    pub fn from_year_month(year: YearType, month: MonthType) -> Self {
        Self(YyyyMmDdDateValue::from(year) * 10_000 + YyyyMmDdDateValue::from(month) * 100)
    }

    #[must_use]
    pub fn is_year(self) -> bool {
        Self::from_year(self.year()) == self
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum YyyyMmDdDateInvalidity {
    Min,
    Max,
    MonthOutOfRange,
    DayOfMonthOutOfRange,
    DayWithoutMonth,
    Invalid,
}

impl Validate for YyyyMmDdDate {
    type Invalidity = YyyyMmDdDateInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::MIN, Self::Invalidity::Min)
            .invalidate_if(*self > Self::MAX, Self::Invalidity::Max)
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

impl fmt::Display for YyyyMmDdDate {
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
                return date.format(YYYY_MM_DD_FORMAT).expect("valid date").fmt(f);
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateOrDateTime {
    Date(YyyyMmDdDate),
    DateTime(OffsetDateTimeMs),
}

impl DateOrDateTime {
    #[must_use]
    pub const fn year(&self) -> YearType {
        match self {
            Self::Date(inner) => inner.year(),
            Self::DateTime(inner) => inner.year(),
        }
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

impl From<OffsetDateTimeMs> for DateOrDateTime {
    fn from(from: OffsetDateTimeMs) -> Self {
        Self::DateTime(from)
    }
}

impl From<YyyyMmDdDate> for DateOrDateTime {
    fn from(from: YyyyMmDdDate) -> Self {
        Self::Date(from)
    }
}

impl From<&DateOrDateTime> for YyyyMmDdDate {
    fn from(from: &DateOrDateTime) -> Self {
        match from {
            DateOrDateTime::Date(date) => *date,
            DateOrDateTime::DateTime(dt) => Self::from_date(dt.date()),
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
            Self::Date(date) => date.fmt(f),
            Self::DateTime(datetime) => datetime.fmt(f),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DateOrDateTimeInvalidity {
    Date(YyyyMmDdDateInvalidity),
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
