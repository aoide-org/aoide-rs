// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::audio::sample::*;
}

///////////////////////////////////////////////////////////////////////
// SamplePosition
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SamplePosition(_core::SamplePositionType);

impl From<_core::SamplePosition> for SamplePosition {
    fn from(from: _core::SamplePosition) -> Self {
        let _core::SamplePosition(sample) = from;
        Self(sample)
    }
}

impl From<SamplePosition> for _core::SamplePosition {
    fn from(from: SamplePosition) -> Self {
        let SamplePosition(sample) = from;
        Self(sample)
    }
}
