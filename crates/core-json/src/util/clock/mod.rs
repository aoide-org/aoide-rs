// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

use semval::Validate as _;
use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};

use aoide_core::util::clock::{YearType, YYYYMMDD};

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::util::clock::{DateOrDateTime, DateTime, DateYYYYMMDD};
}

///////////////////////////////////////////////////////////////////////
// DateTime
///////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq, Eq)]
pub struct DateTime(_core::DateTime);

impl From<_core::DateTime> for DateTime {
    fn from(from: _core::DateTime) -> Self {
        Self(from)
    }
}

impl From<DateTime> for _core::DateTime {
    fn from(from: DateTime) -> Self {
        let DateTime(inner) = from;
        inner
    }
}

#[cfg(feature = "schemars")]
impl JsonSchema for DateTime {
    fn schema_name() -> String {
        "DateTime".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        // TODO: Use predefined schema for `time::OffsetDateTime` when available
        //gen.subschema_for::<time::OffsetDateTime>()
        gen.subschema_for::<String>()
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        time::serde::rfc3339::serialize(self.0.as_ref(), serializer)
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<DateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        time::serde::rfc3339::deserialize(deserializer)
            .map(_core::DateTime::new)
            .map(Into::into)
    }
}

///////////////////////////////////////////////////////////////////////
// DateYYYYMMDD
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[allow(clippy::upper_case_acronyms)]
pub struct DateYYYYMMDD(_core::DateYYYYMMDD);

impl From<_core::DateYYYYMMDD> for DateYYYYMMDD {
    fn from(from: _core::DateYYYYMMDD) -> Self {
        Self(from)
    }
}

impl From<DateYYYYMMDD> for _core::DateYYYYMMDD {
    fn from(from: DateYYYYMMDD) -> Self {
        from.0
    }
}

#[cfg(feature = "schemars")]
impl JsonSchema for DateYYYYMMDD {
    fn schema_name() -> String {
        "DateYYYYMMDD".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        gen.subschema_for::<YYYYMMDD>()
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for DateYYYYMMDD {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = if self.0.is_year() {
            i32::from(self.0.year())
        } else {
            self.0.into()
        };
        serializer.serialize_i32(value)
    }
}

#[allow(clippy::upper_case_acronyms)]
struct DateYYYYMMDDDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for DateYYYYMMDDDeserializeVisitor {
    type Value = DateYYYYMMDD;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("4-digit YYYY or 8-digit YYYYMMDD integer"))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = value as YYYYMMDD;
        let value = if value < _core::DateYYYYMMDD::min().into()
            && value >= YYYYMMDD::from(_core::DateYYYYMMDD::min().year())
            && value <= YYYYMMDD::from(_core::DateYYYYMMDD::max().year())
        {
            // Special case handling: YYYY
            _core::DateYYYYMMDD::from_year(value as YearType)
        } else {
            _core::DateYYYYMMDD::new(value)
        };
        value
            .validate()
            .map_err(|err| E::custom(format!("{err:?}")))
            .map(|()| DateYYYYMMDD(value))
    }
}

impl<'de> Deserialize<'de> for DateYYYYMMDD {
    fn deserialize<D>(deserializer: D) -> Result<DateYYYYMMDD, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(DateYYYYMMDDDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// DateOrDateTime
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(untagged)]
pub enum DateOrDateTime {
    Date(DateYYYYMMDD),
    DateTime(DateTime),
}

impl From<_core::DateOrDateTime> for DateOrDateTime {
    fn from(from: _core::DateOrDateTime) -> Self {
        use _core::DateOrDateTime::*;
        match from {
            Date(from) => Self::Date(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
        }
    }
}

impl From<DateOrDateTime> for _core::DateOrDateTime {
    fn from(from: DateOrDateTime) -> Self {
        use DateOrDateTime::*;
        match from {
            Date(from) => Self::Date(from.into()),
            DateTime(from) => Self::DateTime(from.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
