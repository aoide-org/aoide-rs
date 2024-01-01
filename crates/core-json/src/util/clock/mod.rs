// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use aoide_core::{
    prelude::*,
    util::clock::{YearType, YyyyMmDdDateValue},
};
use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::util::clock::{DateOrDateTime, OffsetDateTimeMs, YyyyMmDdDate};
}

///////////////////////////////////////////////////////////////////////
// DateTime
///////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct DateTime {
    #[cfg_attr(
        feature = "json-schema",
        schemars(with = "chrono::DateTime<chrono::FixedOffset>")
    )]
    inner: _core::OffsetDateTimeMs,
}

impl From<_core::OffsetDateTimeMs> for DateTime {
    fn from(inner: _core::OffsetDateTimeMs) -> Self {
        Self { inner }
    }
}

impl From<DateTime> for _core::OffsetDateTimeMs {
    fn from(from: DateTime) -> Self {
        let DateTime { inner } = from;
        inner
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        time::serde::rfc3339::serialize(self.inner.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<DateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        time::serde::rfc3339::deserialize(deserializer)
            .map(_core::OffsetDateTimeMs::clamp_from)
            .map(Into::into)
    }
}

///////////////////////////////////////////////////////////////////////
// YyyyMmDdDate
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(transparent)]
#[allow(clippy::upper_case_acronyms)]
pub struct YyyyMmDdDate(_core::YyyyMmDdDate);

impl From<_core::YyyyMmDdDate> for YyyyMmDdDate {
    fn from(from: _core::YyyyMmDdDate) -> Self {
        Self(from)
    }
}

impl From<YyyyMmDdDate> for _core::YyyyMmDdDate {
    fn from(from: YyyyMmDdDate) -> Self {
        from.0
    }
}

#[cfg(feature = "json-schema")]
impl schemars::JsonSchema for YyyyMmDdDate {
    fn schema_name() -> String {
        "YyyyMmDdDate".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<YyyyMmDdDateValue>()
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for YyyyMmDdDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = if self.0.is_year() {
            i32::from(self.0.year())
        } else {
            self.0.value()
        };
        serializer.serialize_i32(value)
    }
}

#[allow(clippy::upper_case_acronyms)]
struct YyyyMmDdDateDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for YyyyMmDdDateDeserializeVisitor {
    type Value = YyyyMmDdDate;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "4-digit YYYY or 8-digit YyyyMmDdDateValue integer")
    }

    #[allow(clippy::cast_possible_truncation)]
    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = value as YyyyMmDdDateValue;
        let value = if value < _core::YyyyMmDdDate::MIN.value()
            && value >= YyyyMmDdDateValue::from(_core::YyyyMmDdDate::MIN.year())
            && value <= YyyyMmDdDateValue::from(_core::YyyyMmDdDate::MAX.year())
        {
            // Special case handling: YYYY
            _core::YyyyMmDdDate::from_year(value as YearType)
        } else {
            _core::YyyyMmDdDate::new_unchecked(value)
        };
        value
            .validate()
            .map_err(|err| E::custom(format!("{err:?}")))
            .map(|()| YyyyMmDdDate(value))
    }
}

impl<'de> Deserialize<'de> for YyyyMmDdDate {
    fn deserialize<D>(deserializer: D) -> Result<YyyyMmDdDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(YyyyMmDdDateDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// DateOrDateTime
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum DateOrDateTime {
    Date(YyyyMmDdDate),
    DateTime(DateTime),
}

impl From<_core::DateOrDateTime> for DateOrDateTime {
    fn from(from: _core::DateOrDateTime) -> Self {
        use _core::DateOrDateTime as From;
        match from {
            From::Date(from) => Self::Date(from.into()),
            From::DateTime(from) => Self::DateTime(from.into()),
        }
    }
}

impl From<DateOrDateTime> for _core::DateOrDateTime {
    fn from(from: DateOrDateTime) -> Self {
        use DateOrDateTime as From;
        match from {
            From::Date(from) => Self::Date(from.into()),
            From::DateTime(from) => Self::DateTime(from.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
