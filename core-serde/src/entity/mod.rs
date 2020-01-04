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
    pub use aoide_core::entity::*;
}

use aoide_core::util::clock::{TickInstant, TickType, Ticks};

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};

use std::{fmt, str};

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EntityUid(pub _core::EntityUid);

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for EntityUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Avoid creating a temporary string
        let encoded = self.0.encode_to_string();
        serializer.serialize_str(&encoded)
    }
}

struct EntityUidDeserializeVisitor;

impl<'de> SerdeDeserializeVisitor<'de> for EntityUidDeserializeVisitor {
    type Value = EntityUid;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_fmt(format_args!("base58 encoded string"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        _core::EntityUid::decode_from_str(value)
            .map(EntityUid)
            .map_err(|e| E::custom(format!("{:?}", e)))
    }
}

impl<'de> Deserialize<'de> for EntityUid {
    fn deserialize<D>(deserializer: D) -> Result<EntityUid, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(EntityUidDeserializeVisitor)
    }
}

impl From<EntityUid> for _core::EntityUid {
    fn from(from: EntityUid) -> Self {
        from.0
    }
}

impl From<_core::EntityUid> for EntityUid {
    fn from(from: _core::EntityUid) -> Self {
        Self(from)
    }
}

///////////////////////////////////////////////////////////////////////
// EntityRevision
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct EntityRevision(_core::EntityRevisionVersion, TickType);

impl From<EntityRevision> for _core::EntityRevision {
    fn from(from: EntityRevision) -> Self {
        Self {
            ver: from.0,
            ts: TickInstant(Ticks(from.1)),
        }
    }
}

impl From<_core::EntityRevision> for EntityRevision {
    fn from(from: _core::EntityRevision) -> Self {
        Self(from.ver, (from.ts.0).0)
    }
}

///////////////////////////////////////////////////////////////////////
// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct EntityHeader(EntityUid, EntityRevision);

impl From<EntityHeader> for _core::EntityHeader {
    fn from(from: EntityHeader) -> Self {
        Self {
            uid: from.0.into(),
            rev: from.1.into(),
        }
    }
}

impl From<_core::EntityHeader> for EntityHeader {
    fn from(from: _core::EntityHeader) -> Self {
        Self(from.uid.into(), from.rev.into())
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Entity<B>(pub EntityHeader, pub B);
