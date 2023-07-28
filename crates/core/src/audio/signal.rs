// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fmt;

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

pub type BitrateBpsValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct BitrateBps(BitrateBpsValue);

impl BitrateBps {
    pub const UNIT_OF_MEASURE: &str = "bps";

    pub const ZERO: Self = Self(0.0);
    pub const MIN: Self = Self(BitrateBpsValue::MIN_POSITIVE);
    pub const MAX: Self = Self(BitrateBpsValue::MAX);

    #[must_use]
    pub const fn new(value: BitrateBpsValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> BitrateBpsValue {
        let Self(value) = self;
        value
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
        write!(f, "{} {}", self.value(), Self::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SampleRateHzValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct SampleRateHz(SampleRateHzValue);

impl SampleRateHz {
    pub const UNIT_OF_MEASURE: &str = "Hz";

    pub const ZERO: Self = Self(0.0);
    pub const MIN: Self = Self(SampleRateHzValue::MIN_POSITIVE);
    pub const MAX: Self = Self(SampleRateHzValue::MAX);

    #[must_use]
    pub const fn new(value: SampleRateHzValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> SampleRateHzValue {
        let Self(value) = self;
        value
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
        write!(f, "{} {}", self.value(), Self::UNIT_OF_MEASURE)
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
        debug_assert!(self.is_valid());
        let bps = BitrateBpsValue::from(self.channel_layout.channel_count().0)
            * self.sample_rate.0.round()
            * BitrateBpsValue::from(bits_per_sample);
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

pub type LatencyMsValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct LatencyMs(LatencyMsValue);

impl LatencyMs {
    pub const UNIT_OF_MEASURE: &str = "ms";

    const UNITS_PER_SECOND: LatencyMsValue = 1_000.0;

    pub const ZERO: Self = Self(0.0);
    pub const MIN: Self = Self::ZERO;
    pub const MAX: Self = Self(f64::MAX);

    #[must_use]
    pub const fn new(value: LatencyMsValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> LatencyMsValue {
        let Self(value) = self;
        value
    }

    #[must_use]
    pub fn from_samples(sample_length: SampleLength, sample_rate: SampleRateHz) -> LatencyMs {
        debug_assert!(sample_length.is_valid());
        debug_assert!(sample_rate.is_valid());
        Self(
            (sample_length.value() * Self::UNITS_PER_SECOND)
                / sample_rate.value() as LatencyMsValue,
        )
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

pub type LoudnessLufsValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
pub struct LoudnessLufs(LoudnessLufsValue);

// Loudness is measured according to ITU-R BS.1770 in "Loudness Units
// relative to Full Scale" (LUFS) with 1 LU = 1 dB.
// EBU R128 proposes a target level of -23 LUFS while the ReplayGain v2
// specification (RG2) proposes -18 LUFS for achieving similar perceptive
// results compared to ReplayGain v1 (RG1).
impl LoudnessLufs {
    pub const UNIT_OF_MEASURE: &str = "LUFS";

    #[must_use]
    pub const fn new(value: LoudnessLufsValue) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn value(self) -> LoudnessLufsValue {
        let Self(value) = self;
        value
    }

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
