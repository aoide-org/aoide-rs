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

mod _core {
    pub use aoide_core::util::clock::{DateOrDateTime, DateTime, DateYYYYMMDD};
}

use aoide_core::util::clock::{DateTimeInner, YearType, YYYYMMDD};

use schemars::{gen::SchemaGenerator, schema::Schema};
use semval::Validate;
use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};
use std::fmt;

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

impl JsonSchema for DateTime {
    fn schema_name() -> String {
        "DateTime".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.subschema_for::<DateTimeInner>()
    }
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for DateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Avoid creating a temporary string
        let encoded = self.0.to_string();
        serializer.serialize_str(&encoded)
    }
}

struct DateTimeDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for DateTimeDeserializeVisitor {
    type Value = DateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("RFC 3339 date/time string"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .parse::<_core::DateTime>()
            .map(DateTime)
            .map_err(|e| E::custom(format!("{:?}", e)))
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<DateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DateTimeDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// DateYYYYMMDD
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
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

impl JsonSchema for DateYYYYMMDD {
    fn schema_name() -> String {
        "DateYYYYMMDD".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
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
            .map_err(|e| E::custom(format!("{:?}", e)))
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
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
