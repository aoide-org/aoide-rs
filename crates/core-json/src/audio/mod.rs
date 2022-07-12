// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

pub mod channel;
pub mod sample;
pub mod signal;

mod _core {
    pub(super) use aoide_core::audio::*;
}

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(PartialEq))]
pub struct PositionMs(_core::PositionInMilliseconds);

impl From<_core::PositionMs> for PositionMs {
    fn from(from: _core::PositionMs) -> Self {
        let _core::PositionMs(ms) = from;
        Self(ms)
    }
}

impl From<PositionMs> for _core::PositionMs {
    fn from(from: PositionMs) -> Self {
        let PositionMs(ms) = from;
        Self(ms)
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(PartialEq))]
pub struct DurationMs(_core::DurationInMilliseconds);

impl From<_core::DurationMs> for DurationMs {
    fn from(from: _core::DurationMs) -> Self {
        Self(from.to_inner())
    }
}

impl From<DurationMs> for _core::DurationMs {
    fn from(from: DurationMs) -> Self {
        let DurationMs(ms) = from;
        Self::from_inner(ms)
    }
}
