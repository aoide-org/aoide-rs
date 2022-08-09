// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, str};

use serde::{
    de::{self, Visitor as SerdeDeserializeVisitor},
    Deserializer, Serializer,
};

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::entity::*;
}

///////////////////////////////////////////////////////////////////////
// EntityUid
///////////////////////////////////////////////////////////////////////

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct EntityUid {
    #[cfg_attr(feature = "json-schema", schemars(with = "String"))]
    inner: _core::EntityUid,
}

// Serialize (and deserialize) as string for maximum compatibility and portability
impl Serialize for EntityUid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // TODO: Avoid creating a temporary string
        let encoded = self.inner.encode_to_string();
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
            .map(|inner| EntityUid { inner })
            .map_err(<E as de::Error>::custom)
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
        let Self { inner } = self;
        inner
    }
}

impl From<EntityUid> for _core::EntityUid {
    fn from(from: EntityUid) -> Self {
        let EntityUid { inner } = from;
        inner
    }
}

impl From<_core::EntityUid> for EntityUid {
    fn from(inner: _core::EntityUid) -> Self {
        Self { inner }
    }
}

impl<T> From<EntityUid> for _core::EntityUidTyped<T> {
    fn from(from: EntityUid) -> Self {
        Self::from_untyped(from.into())
    }
}

impl<T> From<_core::EntityUidTyped<T>> for EntityUid {
    fn from(from: _core::EntityUidTyped<T>) -> Self {
        from.into_untyped().into()
    }
}

///////////////////////////////////////////////////////////////////////
// EntityRevision
///////////////////////////////////////////////////////////////////////

pub type EntityRevision = _core::EntityRevision;

///////////////////////////////////////////////////////////////////////
// EntityHeader
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct EntityHeader(EntityUid, EntityRevision);

impl From<EntityHeader> for _core::EntityHeader {
    fn from(from: EntityHeader) -> Self {
        let EntityHeader(uid, rev) = from;
        Self {
            uid: uid.into(),
            rev,
        }
    }
}

impl<T> From<EntityHeader> for _core::EntityHeaderTyped<T> {
    fn from(from: EntityHeader) -> Self {
        _core::EntityHeaderTyped::from_untyped(_core::EntityHeader::from(from))
    }
}

impl From<_core::EntityHeader> for EntityHeader {
    fn from(from: _core::EntityHeader) -> Self {
        let _core::EntityHeader { uid, rev } = from;
        Self(uid.into(), rev)
    }
}

impl<T> From<_core::EntityHeaderTyped<T>> for EntityHeader {
    fn from(from: _core::EntityHeaderTyped<T>) -> Self {
        from.into_untyped().into()
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Entity<B>(pub EntityHeader, pub B);
