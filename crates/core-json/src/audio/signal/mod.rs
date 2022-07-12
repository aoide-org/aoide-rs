// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::audio::signal::*;
}

///////////////////////////////////////////////////////////////////////
// Bitrate
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct BitrateBps(_core::BitsPerSecond);

impl From<_core::BitrateBps> for BitrateBps {
    fn from(from: _core::BitrateBps) -> Self {
        Self(from.to_inner())
    }
}

impl From<BitrateBps> for _core::BitrateBps {
    fn from(from: BitrateBps) -> Self {
        Self::from_inner(from.0)
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct SampleRateHz(_core::SamplesPerSecond);

impl From<_core::SampleRateHz> for SampleRateHz {
    fn from(from: _core::SampleRateHz) -> Self {
        Self(from.to_inner())
    }
}

impl From<SampleRateHz> for _core::SampleRateHz {
    fn from(from: SampleRateHz) -> Self {
        let SampleRateHz(inner) = from;
        Self::from_inner(inner)
    }
}

///////////////////////////////////////////////////////////////////////
// Loudness
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
pub struct LoudnessLufs(_core::LufsValue);

impl From<_core::LoudnessLufs> for LoudnessLufs {
    fn from(from: _core::LoudnessLufs) -> Self {
        let _core::LoudnessLufs(lufs) = from;
        Self(lufs)
    }
}

impl From<LoudnessLufs> for _core::LoudnessLufs {
    fn from(from: LoudnessLufs) -> Self {
        let LoudnessLufs(lufs) = from;
        Self(lufs)
    }
}
