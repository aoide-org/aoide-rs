// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::music::key::KeyCodeValue;

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::music::key::KeyCode;
}

/// The key signature is denoted by a code with one of the following values:
///
/// * 0 = Off key
/// * 1 = C maj
/// * 2 = A min
/// * 3 = G maj
/// * 4 = E min
/// * ...
/// * 23 = F maj
/// * 24 = D min
///
/// The numbering follows the Circle of fifth / Open Key notation in clock-wise orientation,
/// alternating between major and minor keys.
#[derive(Debug, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(transparent)
)]
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
