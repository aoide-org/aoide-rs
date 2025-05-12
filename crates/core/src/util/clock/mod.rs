// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{convert::Infallible, fmt, str::FromStr};

use anyhow::anyhow;
use jiff::{
    Timestamp, Zoned,
    civil::{Date, DateTime, Time},
    fmt::temporal::Pieces,
    tz::{Offset, TimeZone},
};
use semval::prelude::*;

pub type TimestampMillis = i64;

/// An _UTC_ timestamp with truncated millisecond precision.
#[derive(Clone, Debug, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct UtcDateTimeMs {
    unix_timestamp_millis: TimestampMillis,
}

impl UtcDateTimeMs {
    #[must_use]
    pub const fn from_unix_timestamp_millis(unix_timestamp_millis: TimestampMillis) -> Self {
        Self {
            unix_timestamp_millis,
        }
    }

    #[must_use]
    pub const fn unix_timestamp_millis(&self) -> TimestampMillis {
        self.unix_timestamp_millis
    }

    #[must_use]
    pub fn from_timestamp(timestamp: &Timestamp) -> Self {
        Self::from_unix_timestamp_millis(timestamp.as_millisecond())
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc, reason = "should never panic")]
    pub fn to_timestamp(&self) -> Timestamp {
        Timestamp::from_millisecond(self.unix_timestamp_millis).expect("valid timestamp")
    }

    #[must_use]
    pub fn now() -> Self {
        Self::from_unix_timestamp_millis(Timestamp::now().as_millisecond())
    }
}

impl From<Timestamp> for UtcDateTimeMs {
    fn from(from: Timestamp) -> Self {
        Self::from_timestamp(&from)
    }
}

impl From<UtcDateTimeMs> for Timestamp {
    fn from(from: UtcDateTimeMs) -> Self {
        from.to_timestamp()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for UtcDateTimeMs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_timestamp().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for UtcDateTimeMs {
    fn deserialize<D>(deserializer: D) -> Result<UtcDateTimeMs, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Timestamp::deserialize(deserializer).map(|ts| Self::from_timestamp(&ts))
    }
}

/// A timestamp with time zone offset and truncated millisecond precision.
#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct OffsetDateTimeMs {
    utc_date_time: UtcDateTimeMs,
    utc_offset_secs: i32,
}

impl PartialOrd for OffsetDateTimeMs {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let Self {
            utc_date_time,
            utc_offset_secs,
        } = *self;
        if utc_offset_secs != other.utc_offset_secs {
            return None;
        }
        utc_date_time.partial_cmp(&other.utc_date_time)
    }
}

impl From<&Zoned> for OffsetDateTimeMs {
    fn from(from: &Zoned) -> Self {
        let utc_date_time = from.timestamp().into();
        let utc_offset_secs = from.offset().seconds();
        Self {
            utc_date_time,
            utc_offset_secs,
        }
    }
}

impl From<Zoned> for OffsetDateTimeMs {
    fn from(from: Zoned) -> Self {
        From::from(&from)
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
#[cfg(feature = "serde")]
impl serde::Serialize for OffsetDateTimeMs {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(self)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for OffsetDateTimeMs {
    fn deserialize<D>(deserializer: D) -> Result<OffsetDateTimeMs, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(OffsetDateTimeMsDeserializeFromStr)
    }
}

#[cfg(feature = "serde")]
struct OffsetDateTimeMsDeserializeFromStr;

#[cfg(feature = "serde")]
impl serde::de::Visitor<'_> for OffsetDateTimeMsDeserializeFromStr {
    type Value = OffsetDateTimeMs;

    fn visit_str<E>(self, input: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        input
            .parse()
            .map_err(|_| serde::de::Error::invalid_value(serde::de::Unexpected::Str(input), &self))
    }

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a string containing an RFC 3339 timestamp with offset"
        )
    }
}

impl OffsetDateTimeMs {
    #[must_use]
    pub const fn from_utc(utc_date_time: UtcDateTimeMs) -> Self {
        Self {
            utc_date_time,
            utc_offset_secs: 0,
        }
    }

    #[must_use]
    pub fn from_zoned(zoned: &Zoned) -> Self {
        let utc_date_time = zoned.timestamp().into();
        let utc_offset_secs = zoned.offset().seconds();
        Self {
            utc_date_time,
            utc_offset_secs,
        }
    }

    #[must_use]
    fn try_from_pieces(pieces: &Pieces) -> Option<Self> {
        let offset = pieces.to_numeric_offset()?;
        let time = pieces.time().unwrap_or(Time::midnight());
        let date = pieces.date();
        let zoned = DateTime::from_parts(date, time)
            .to_zoned(TimeZone::fixed(offset))
            .ok()?;
        Some(Self::from_zoned(&zoned))
    }

    #[must_use]
    pub const fn from_unix_timestamp_millis(unix_timestamp_millis: TimestampMillis) -> Self {
        Self::from_utc(UtcDateTimeMs::from_unix_timestamp_millis(
            unix_timestamp_millis,
        ))
    }

    #[must_use]
    pub const fn unix_timestamp_millis(&self) -> TimestampMillis {
        self.to_utc().unix_timestamp_millis()
    }

    #[must_use]
    pub fn now_utc() -> Self {
        Self::from_utc(UtcDateTimeMs::now())
    }

    #[must_use]
    pub fn now_local() -> Self {
        Zoned::now().into()
    }

    #[must_use]
    pub const fn to_utc(&self) -> UtcDateTimeMs {
        self.utc_date_time
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc, reason = "Offset is always valid.")]
    pub fn offset(&self) -> Offset {
        Offset::from_seconds(self.utc_offset_secs).expect("valid offset")
    }

    #[must_use]
    pub fn to_timestamp_with_offset(&self) -> (Timestamp, Offset) {
        let offset = self.offset();
        let timestamp = self.utc_date_time.to_timestamp();
        (timestamp, offset)
    }

    #[must_use]
    pub fn to_zoned(&self) -> Zoned {
        let (timestamp, offset) = self.to_timestamp_with_offset();
        let time_zone = if offset.is_zero() {
            TimeZone::UTC
        } else {
            TimeZone::fixed(offset)
        };
        Zoned::new(timestamp, time_zone)
    }

    #[must_use]
    pub fn date(&self) -> Date {
        self.to_zoned().date()
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

impl FromStr for OffsetDateTimeMs {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Pieces::parse(input)
            .map_err(Into::into)
            .and_then(|pcs| Self::try_from_pieces(&pcs).ok_or_else(|| anyhow!("missing pieces")))
    }
}

impl fmt::Display for OffsetDateTimeMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            utc_date_time,
            utc_offset_secs,
        } = self;
        let timestamp = utc_date_time.to_timestamp();
        if *utc_offset_secs == 0 {
            // Zulu offset.
            timestamp.fmt(f)
        } else {
            // Numeric offset.
            let offset = Offset::from_seconds(*utc_offset_secs).expect("valid offset");
            timestamp.display_with_offset(offset).fmt(f)
        }
    }
}

impl Validate for OffsetDateTimeMs {
    type Invalidity = Infallible;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new().into()
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
    pub fn from_date(from: Date) -> Self {
        Self(
            YyyyMmDdDateValue::from(from.year()) * 10_000
                + YyyyMmDdDateValue::from(from.month()) * 100
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
                    && Date::new(self.year(), self.month(), self.day_of_month()).is_err(),
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
            if let Ok(date) = Date::new(self.year(), self.month(), self.day_of_month()) {
                return date.fmt(f);
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DateOrDateTime {
    Date(YyyyMmDdDate),
    DateTime(OffsetDateTimeMs),
}

impl DateOrDateTime {
    #[must_use]
    pub fn year(&self) -> YearType {
        match self {
            Self::Date(inner) => inner.year(),
            Self::DateTime(inner) => inner.date().year() as _,
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
            DateOrDateTime::Date(inner) => *inner,
            DateOrDateTime::DateTime(inner) => Self::from_date(inner.date()),
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
