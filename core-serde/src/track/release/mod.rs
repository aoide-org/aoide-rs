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

mod _core {
    pub use aoide_core::track::release::{Release, ReleaseDate, ReleaseDateTime, ReleasedAt};
}

use aoide_core::track::release::YYYYMMDD;

use semval::Validate;

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};

use std::fmt;

///////////////////////////////////////////////////////////////////////
// ReleaseDateTime
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct ReleaseDate(_core::ReleaseDate);

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for ReleaseDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i32(self.0.into())
    }
}

struct ReleaseDateDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for ReleaseDateDeserializeVisitor {
    type Value = ReleaseDate;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("4-digit YYYY or 8-digit YYYYMMDD integer"))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut value = value as YYYYMMDD;
        if value < _core::ReleaseDate::min().into()
            && value >= YYYYMMDD::from(_core::ReleaseDate::min().year())
            && value <= YYYYMMDD::from(_core::ReleaseDate::max().year())
        {
            // Special case handling: YYYY -> YYYY0000
            value *= 10_000;
        }
        let value = _core::ReleaseDate::new(value);
        value
            .validate()
            .map_err(|e| E::custom(format!("{:?}", e)))
            .map(|()| ReleaseDate(value))
    }
}

impl<'de> Deserialize<'de> for ReleaseDate {
    fn deserialize<D>(deserializer: D) -> Result<ReleaseDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(ReleaseDateDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// ReleaseDateTime
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct ReleaseDateTime(_core::ReleaseDateTime);

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for ReleaseDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Avoid creating a temporary string
        let encoded = self.0.to_string();
        serializer.serialize_str(&encoded)
    }
}

struct ReleaseDateTimeDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for ReleaseDateTimeDeserializeVisitor {
    type Value = ReleaseDateTime;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("RFC 3339 date/time string"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        value
            .parse::<_core::ReleaseDateTime>()
            .map(ReleaseDateTime)
            .map_err(|e| E::custom(format!("{:?}", e)))
    }
}

impl<'de> Deserialize<'de> for ReleaseDateTime {
    fn deserialize<D>(deserializer: D) -> Result<ReleaseDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ReleaseDateTimeDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// ReleasedAt
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(untagged)]
pub enum ReleasedAt {
    Date(ReleaseDate),
    DateTime(ReleaseDateTime),
}

impl From<_core::ReleasedAt> for ReleasedAt {
    fn from(from: _core::ReleasedAt) -> Self {
        use _core::ReleasedAt::*;
        match from {
            Date(from) => ReleasedAt::Date(ReleaseDate(from)),
            DateTime(from) => ReleasedAt::DateTime(ReleaseDateTime(from)),
        }
    }
}

impl From<ReleasedAt> for _core::ReleasedAt {
    fn from(from: ReleasedAt) -> Self {
        use _core::ReleasedAt::*;
        match from {
            ReleasedAt::Date(from) => Date(from.0),
            ReleasedAt::DateTime(from) => DateTime(from.0),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Release
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Release {
    #[serde(rename = "own", skip_serializing_if = "Option::is_none")]
    released_by: Option<String>,

    #[serde(rename = "dat", skip_serializing_if = "Option::is_none")]
    released_at: Option<ReleasedAt>,

    #[serde(rename = "cpy", skip_serializing_if = "Option::is_none")]
    copyright: Option<String>,

    #[serde(rename = "lic", skip_serializing_if = "Vec::is_empty", default)]
    licenses: Vec<String>,
}

impl From<_core::Release> for Release {
    fn from(from: _core::Release) -> Self {
        let _core::Release {
            released_at,
            released_by,
            copyright,
            licenses,
        } = from;
        Self {
            released_at: released_at.map(Into::into),
            released_by,
            copyright,
            licenses,
        }
    }
}

impl From<Release> for _core::Release {
    fn from(from: Release) -> Self {
        let Release {
            released_at,
            released_by,
            copyright,
            licenses,
        } = from;
        Self {
            released_at: released_at.map(Into::into),
            released_by,
            copyright,
            licenses,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
