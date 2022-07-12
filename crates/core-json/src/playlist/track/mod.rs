// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use crate::entity::EntityUid;

mod _core {
    pub(super) use aoide_core::playlist::track::*;
}

///////////////////////////////////////////////////////////////////////
// Item
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Item {
    pub(crate) uid: EntityUid,
}

impl From<Item> for _core::Item {
    fn from(from: Item) -> Self {
        let Item { uid } = from;
        Self { uid: uid.into() }
    }
}

impl From<_core::Item> for Item {
    fn from(from: _core::Item) -> Self {
        let _core::Item { uid } = from;
        Self { uid: uid.into() }
    }
}
