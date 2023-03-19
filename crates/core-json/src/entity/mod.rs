// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::{EntityHeader, EntityHeaderTyped};
}

pub use aoide_core::{EntityRevision, EntityUid};

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
        Self { uid, rev }
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
        Self(uid, rev)
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
