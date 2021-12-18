// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
    pub use aoide_core::entity::*;
}

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};

use schemars::{gen::SchemaGenerator, schema::Schema};
use std::{fmt, str};

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct EntityUid(_core::EntityUid);

impl JsonSchema for EntityUid {
    fn schema_name() -> String {
        "EntityUid".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        gen.subschema_for::<String>()
    }
}

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

impl AsRef<_core::EntityUid> for EntityUid {
    fn as_ref(&self) -> &_core::EntityUid {
        let Self(inner) = self;
        inner
    }
}

impl From<EntityUid> for _core::EntityUid {
    fn from(from: EntityUid) -> Self {
        let EntityUid(inner) = from;
        inner
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

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct EntityRevision(_core::EntityRevisionNumber);

impl From<EntityRevision> for _core::EntityRevision {
    fn from(from: EntityRevision) -> Self {
        let EntityRevision(number) = from;
        Self::from_inner(number)
    }
}

impl From<_core::EntityRevision> for EntityRevision {
    fn from(from: _core::EntityRevision) -> Self {
        Self(from.to_inner())
    }
}

///////////////////////////////////////////////////////////////////////
// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct EntityHeader(EntityUid, EntityRevision);

impl From<EntityHeader> for _core::EntityHeader {
    fn from(from: EntityHeader) -> Self {
        let EntityHeader(uid, rev) = from;
        Self {
            uid: uid.into(),
            rev: rev.into(),
        }
    }
}

impl From<_core::EntityHeader> for EntityHeader {
    fn from(from: _core::EntityHeader) -> Self {
        let _core::EntityHeader { uid, rev } = from;
        Self(uid.into(), rev.into())
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct Entity<B>(pub EntityHeader, pub B);
