// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::music::key::KeyCodeValue;

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::music::key::KeyCode;
}

#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct KeyCode(KeyCodeValue);

impl From<_core::KeyCode> for KeyCode {
    fn from(from: _core::KeyCode) -> Self {
        Self(from.to_value())
    }
}

impl TryFrom<KeyCode> for _core::KeyCode {
    type Error = ();

    fn try_from(from: KeyCode) -> Result<Self, Self::Error> {
        let KeyCode(val) = from;
        Self::try_from(val)
    }
}
