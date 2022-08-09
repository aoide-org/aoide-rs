// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{f64, fmt};

use crate::{
    audio::{
        channel::{ChannelLayout, ChannelLayoutInvalidity},
        sample::BitsPerSample,
        sample::{SampleLayout, SampleLayoutInvalidity, SampleLength},
    },
    prelude::*,
};

///////////////////////////////////////////////////////////////////////
// Bitrate
///////////////////////////////////////////////////////////////////////

pub type BitsPerSecond = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct BitrateBps(BitsPerSecond);

impl BitrateBps {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "bps"
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(f64::MIN_POSITIVE)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(f64::MAX)
    }

    #[must_use]
    pub const fn from_inner(inner: SamplesPerSecond) -> Self {
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
            .invalidate_if(*self < Self::min(), Self::Invalidity::Min(Self::min()))
            .invalidate_if(*self > Self::max(), Self::Invalidity::Max(Self::max()))
            .into()
    }
}

impl fmt::Display for BitrateBps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_inner(), Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SamplesPerSecond = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct SampleRateHz(SamplesPerSecond);

impl SampleRateHz {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "Hz"
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(f64::MIN_POSITIVE)
    }

    #[must_use]
    pub const fn max() -> Self {
        Self(192_000.0)
    }

    #[must_use]
    pub const fn of_compact_disc() -> Self {
        Self(44_100.0)
    }

    #[must_use]
    pub const fn of_studio_48k() -> Self {
        Self(48_000.0)
    }

    #[must_use]
    pub const fn of_studio_96k() -> Self {
        Self(96_000.0)
    }

    #[must_use]
    pub const fn of_studio_192k() -> Self {
        Self(192_000.0)
    }

    #[must_use]
    pub const fn from_inner(inner: SamplesPerSecond) -> Self {
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
            .invalidate_if(*self < Self::min(), Self::Invalidity::Min(Self::min()))
            .invalidate_if(*self > Self::max(), Self::Invalidity::Max(Self::max()))
            .into()
    }
}

impl fmt::Display for SampleRateHz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_inner(), Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// PcmSignal
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PcmSignal {
    pub channel_layout: ChannelLayout,

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
    ChannelLayout(ChannelLayoutInvalidity),
    SampleLayout(SampleLayoutInvalidity),
    SampleRate(SampleRateHzInvalidity),
}

impl Validate for PcmSignal {
    type Invalidity = PcmSignalInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.channel_layout, Self::Invalidity::ChannelLayout)
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct LatencyMs(pub LatencyInMilliseconds);

impl LatencyMs {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    const fn units_per_second() -> LatencyInMilliseconds {
        1_000.0
    }

    #[must_use]
    pub const fn min() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub fn from_samples(sample_length: SampleLength, sample_rate: SampleRateHz) -> LatencyMs {
        debug_assert!(sample_length.validate().is_ok());
        debug_assert!(sample_rate.validate().is_ok());
        Self((sample_length.0 * Self::units_per_second()) / sample_rate.0)
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
                !self.0.is_finite() || *self < Self::min(),
                Self::Invalidity::OutOfRange,
            )
            .into()
    }
}

impl fmt::Display for LatencyMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Loudness
///////////////////////////////////////////////////////////////////////

pub type LufsValue = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(transparent))]
#[repr(transparent)]
pub struct LoudnessLufs(pub LufsValue);

// Loudness is measured according to ITU-R BS.1770 in "Loudness Units
// relative to Full Scale" (LUFS) with 1 LU = 1 dB.
// EBU R128 proposes a target level of -23 LUFS while the ReplayGain v2
// specification (RG2) proposes -18 LUFS for achieving similar perceptive
// results compared to ReplayGain v1 (RG1).
impl LoudnessLufs {
    #[must_use]
    pub const fn unit_of_measure() -> &'static str {
        "LUFS"
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
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
