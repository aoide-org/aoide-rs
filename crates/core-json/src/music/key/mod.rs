// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

use aoide_core::music::key::KeyCodeValue;

mod _core {
    pub(super) use aoide_core::music::key::KeyCode;
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[repr(transparent)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
