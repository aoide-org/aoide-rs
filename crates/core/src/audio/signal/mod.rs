// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{f64, fmt};

use crate::{
    audio::{
        channel::{ChannelFlags, ChannelFlagsInvalidity},
        sample::{BitsPerSample, SampleLayout, SampleLayoutInvalidity, SampleLength},
    },
    prelude::*,
};

///////////////////////////////////////////////////////////////////////
// Bitrate
///////////////////////////////////////////////////////////////////////

pub type BitsPerSecond = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct BitrateBps(BitsPerSecond);

impl BitrateBps {
    pub const UNIT_OF_MEASURE: &str = "bps";

    pub const MIN: Self = Self(f64::MIN_POSITIVE);
    pub const MAX: Self = Self(f64::MAX);

    #[must_use]
    pub const fn new(inner: SamplesPerSecond) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn to_inner(self) -> SamplesPerSecond {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum BitrateBpsInvalidity {
    Min(BitrateBps),
    Max(BitrateBps),
}

impl Validate for BitrateBps {
    type Invalidity = BitrateBpsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::MIN, Self::Invalidity::Min(Self::MIN))
            .invalidate_if(*self > Self::MAX, Self::Invalidity::Max(Self::MAX))
            .into()
    }
}

impl fmt::Display for BitrateBps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_inner(), Self::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SamplesPerSecond = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct SampleRateHz(SamplesPerSecond);

impl SampleRateHz {
    pub const UNIT_OF_MEASURE: &str = "Hz";

    pub const MIN: Self = Self(f64::MIN_POSITIVE);
    pub const MAX: Self = Self(192_000.0);

    #[must_use]
    pub const fn new(inner: SamplesPerSecond) -> Self {
        Self(inner)
    }

    #[must_use]
    pub const fn to_inner(self) -> SamplesPerSecond {
        let Self(inner) = self;
        inner
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum SampleRateHzInvalidity {
    Min(SampleRateHz),
    Max(SampleRateHz),
}

impl Validate for SampleRateHz {
    type Invalidity = SampleRateHzInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::MIN, Self::Invalidity::Min(Self::MIN))
            .invalidate_if(*self > Self::MAX, Self::Invalidity::Max(Self::MAX))
            .into()
    }
}

impl fmt::Display for SampleRateHz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_inner(), Self::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
// PcmSignal
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PcmSignal {
    pub channel_layout: ChannelFlags,

    pub sample_layout: SampleLayout,

    pub sample_rate: SampleRateHz,
}

impl PcmSignal {
    #[must_use]
    pub fn bitrate(self, bits_per_sample: BitsPerSample) -> BitrateBps {
        debug_assert!(self.validate().is_ok());
        let bps = BitsPerSecond::from(self.channel_layout.channel_count().0)
            * (self.sample_rate.0.round() as BitsPerSecond)
            * BitsPerSecond::from(bits_per_sample);
        BitrateBps(bps)
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PcmSignalInvalidity {
    ChannelFlags(ChannelFlagsInvalidity),
    SampleLayout(SampleLayoutInvalidity),
    SampleRate(SampleRateHzInvalidity),
}

impl Validate for PcmSignal {
    type Invalidity = PcmSignalInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.channel_layout, Self::Invalidity::ChannelFlags)
            .validate_with(&self.sample_layout, Self::Invalidity::SampleLayout)
            .validate_with(&self.sample_rate, Self::Invalidity::SampleRate)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Latency
///////////////////////////////////////////////////////////////////////

pub type LatencyInMilliseconds = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct LatencyMs(pub LatencyInMilliseconds);

impl LatencyMs {
    pub const UNIT_OF_MEASURE: &str = "ms";

    const UNITS_PER_SECOND: LatencyInMilliseconds = 1_000.0;

    pub const MIN: Self = Self(0.0);
    pub const MAX: Self = Self(192_000.0);

    #[must_use]
    pub fn from_samples(sample_length: SampleLength, sample_rate: SampleRateHz) -> LatencyMs {
        debug_assert!(sample_length.validate().is_ok());
        debug_assert!(sample_rate.validate().is_ok());
        Self((sample_length.0 * Self::UNITS_PER_SECOND) / sample_rate.0)
    }

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LatencyMsInvalidity {
    OutOfRange,
}

impl Validate for LatencyMs {
    type Invalidity = LatencyMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !self.0.is_finite() || *self < Self::MIN,
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for LatencyMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
// Loudness
///////////////////////////////////////////////////////////////////////

pub type LufsValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct LoudnessLufs(pub LufsValue);

// Loudness is measured according to ITU-R BS.1770 in "Loudness Units
// relative to Full Scale" (LUFS) with 1 LU = 1 dB.
// EBU R128 proposes a target level of -23 LUFS while the ReplayGain v2
// specification (RG2) proposes -18 LUFS for achieving similar perceptive
// results compared to ReplayGain v1 (RG1).
impl LoudnessLufs {
    pub const UNIT_OF_MEASURE: &str = "LUFS";

    #[must_use]
    pub fn is_valid(&self) -> bool {
        <Self as IsValid>::is_valid(self)
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LoudnessLufsInvalidity {
    OutOfRange,
}

impl Validate for LoudnessLufs {
    type Invalidity = LoudnessLufsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.0.is_finite(), Self::Invalidity::OutOfRange)
            .into()
    }
}

impl fmt::Display for LoudnessLufs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
