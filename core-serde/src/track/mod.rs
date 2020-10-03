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

pub mod album;
pub mod extra;
pub mod index;
pub mod marker;
pub mod music;
pub mod release;

use self::{album::*, extra::*, index::*, marker::*, music::*, release::*};

use crate::{
    actor::*, collection::SingleTrackEntry as CollectionSingleTrackEntry, media, tag::*, title::*,
};

mod _core {
    pub use aoide_core::track::*;
}

use aoide_core::track::YYYYMMDD;

use aoide_core::util::IsDefault;

use semval::Validate;
use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};
use std::fmt;

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Track {
    #[serde(rename = "src", skip_serializing_if = "IsDefault::is_default", default)]
    pub media_sources: Vec<media::Source>,

    #[serde(rename = "msc", skip_serializing_if = "IsDefault::is_default", default)]
    pub musical_signature: MusicalSignature,

    #[serde(rename = "rel", skip_serializing_if = "IsDefault::is_default", default)]
    pub release: Release,

    #[serde(rename = "alb", skip_serializing_if = "IsDefault::is_default", default)]
    pub album: Album,

    #[serde(rename = "tit", skip_serializing_if = "IsDefault::is_default", default)]
    pub titles: Vec<Title>,

    #[serde(rename = "act", skip_serializing_if = "IsDefault::is_default", default)]
    pub actors: Vec<Actor>,

    #[serde(rename = "idx", skip_serializing_if = "IsDefault::is_default", default)]
    pub indexes: Indexes,

    #[serde(rename = "mrk", skip_serializing_if = "IsDefault::is_default", default)]
    pub markers: Markers,

    #[serde(rename = "tag", skip_serializing_if = "IsDefault::is_default", default)]
    pub tags: Tags,

    #[serde(rename = "ext", skip_serializing_if = "IsDefault::is_default", default)]
    pub extra: Extra,
}

impl From<_core::Track> for Track {
    fn from(from: _core::Track) -> Self {
        let _core::Track {
            media_sources,
            musical_signature,
            release,
            album,
            titles,
            actors,
            indexes,
            markers,
            tags,
            extra,
        } = from;
        Self {
            media_sources: media_sources.into_iter().map(Into::into).collect(),
            musical_signature: musical_signature.into(),
            release: release.into(),
            album: album.into(),
            titles: titles.into_iter().map(Into::into).collect(),
            actors: actors.into_iter().map(Into::into).collect(),
            indexes: indexes.into(),
            markers: markers.into(),
            tags: tags.into(),
            extra: extra.into(),
        }
    }
}

impl From<Track> for _core::Track {
    fn from(from: Track) -> Self {
        let Track {
            media_sources,
            musical_signature,
            release,
            album,
            titles,
            actors,
            indexes,
            markers,
            tags,
            extra,
        } = from;
        Self {
            media_sources: media_sources.into_iter().map(Into::into).collect(),
            musical_signature: musical_signature.into(),
            release: release.into(),
            album: album.into(),
            titles: titles.into_iter().map(Into::into).collect(),
            actors: actors.into_iter().map(Into::into).collect(),
            indexes: indexes.into(),
            markers: markers.into(),
            tags: tags.into(),
            extra: extra.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

pub type Entity = crate::entity::Entity<Track>;

impl From<Entity> for _core::Entity {
    fn from(from: Entity) -> Self {
        Self::new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct EntityInCollection(pub Entity, pub CollectionSingleTrackEntry);

///////////////////////////////////////////////////////////////////////
// DateTime
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Date(_core::Date);

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for Date {
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

struct DateDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for DateDeserializeVisitor {
    type Value = Date;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("4-digit YYYY or 8-digit YYYYMMDD integer"))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let value = value as YYYYMMDD;
        let value = if value < _core::Date::min().into()
            && value >= YYYYMMDD::from(_core::Date::min().year())
            && value <= YYYYMMDD::from(_core::Date::max().year())
        {
            // Special case handling: YYYY
            _core::Date::from_year(value as _core::YearType)
        } else {
            _core::Date::new(value)
        };
        value
            .validate()
            .map_err(|e| E::custom(format!("{:?}", e)))
            .map(|()| Date(value))
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u64(DateDeserializeVisitor)
    }
}

///////////////////////////////////////////////////////////////////////
// DateTime
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct DateTime(_core::DateTime);

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
// DateOrDateTime
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DateOrDateTime {
    Date(Date),
    DateTime(DateTime),
}

impl From<_core::DateOrDateTime> for DateOrDateTime {
    fn from(from: _core::DateOrDateTime) -> Self {
        match from {
            _core::DateOrDateTime::Date(from) => DateOrDateTime::Date(Date(from)),
            _core::DateOrDateTime::DateTime(from) => DateOrDateTime::DateTime(DateTime(from)),
        }
    }
}

impl From<DateOrDateTime> for _core::DateOrDateTime {
    fn from(from: DateOrDateTime) -> Self {
        use _core::DateOrDateTime::*;
        match from {
            DateOrDateTime::Date(from) => Date(from.0),
            DateOrDateTime::DateTime(from) => DateTime(from.0),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
