// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::*;

use crate::audio::sample::BitsPerSample;

use std::{fmt, u32};

///////////////////////////////////////////////////////////////////////
// BitRate
///////////////////////////////////////////////////////////////////////

pub type BitsPerSecond = u32;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct BitRateBps(pub BitsPerSecond);

impl BitRateBps {
    pub const fn unit_of_measure() -> &'static str {
        "bps"
    }

    pub const fn min() -> Self {
        Self(1)
    }

    pub const fn max() -> Self {
        Self(u32::MAX)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BitRateBpsInvalidity {
    Min(BitRateBps),
    Max(BitRateBps),
}

impl Validate for BitRateBps {
    type Invalidity = BitRateBpsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(*self < Self::min(), BitRateBpsInvalidity::Min(Self::min()))
            .invalidate_if(*self > Self::max(), BitRateBpsInvalidity::Max(Self::max()))
            .into()
    }
}

impl fmt::Display for BitRateBps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SamplesPerSecond = u32;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct SampleRateHz(pub SamplesPerSecond);

impl SampleRateHz {
    pub const fn unit_of_measure() -> &'static str {
        "Hz"
    }

    pub const fn min() -> Self {
        Self(1)
    }

    pub const fn max() -> Self {
        Self(u32::MAX)
    }

    pub const fn of_compact_disc() -> Self {
        Self(44_100)
    }

    pub const fn of_studio_48k() -> Self {
        Self(48_000)
    }

    pub const fn of_studio_96k() -> Self {
        Self(96_000)
    }

    pub const fn of_studio_192k() -> Self {
        Self(192_000)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SampleRateHzInvalidity {
    Min(SampleRateHz),
    Max(SampleRateHz),
}

impl Validate for SampleRateHz {
    type Invalidity = SampleRateHzInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                *self < Self::min(),
                SampleRateHzInvalidity::Min(Self::min()),
            )
            .invalidate_if(
                *self > Self::max(),
                SampleRateHzInvalidity::Max(Self::max()),
            )
            .into()
    }
}

impl fmt::Display for SampleRateHz {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
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
    pub fn bitrate(self, bits_per_sample: BitsPerSample) -> BitRateBps {
        debug_assert!(self.validate().is_ok());
        let bps = BitsPerSecond::from(self.channel_layout.channel_count().0)
            * self.sample_rate.0
            * BitsPerSecond::from(bits_per_sample);
        BitRateBps(bps)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PcmSignalInvalidity {
    ChannelLayout(ChannelLayoutInvalidity),
    SampleLayout(SampleLayoutInvalidity),
    SampleRate(SampleRateHzInvalidity),
}

impl Validate for PcmSignal {
    type Invalidity = PcmSignalInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.channel_layout, PcmSignalInvalidity::ChannelLayout)
            .validate_with(&self.sample_layout, PcmSignalInvalidity::SampleLayout)
            .validate_with(&self.sample_rate, PcmSignalInvalidity::SampleRate)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Latency
///////////////////////////////////////////////////////////////////////

pub type LatencyInMilliseconds = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct LatencyMs(pub LatencyInMilliseconds);

impl LatencyMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    const fn units_per_second() -> LatencyInMilliseconds {
        1_000.0
    }

    pub const fn min() -> Self {
        Self(0.0)
    }

    pub fn from_samples(sample_length: SampleLength, sample_rate: SampleRateHz) -> LatencyMs {
        debug_assert!(sample_length.validate().is_ok());
        debug_assert!(sample_rate.validate().is_ok());
        Self(
            (sample_length.0 * Self::units_per_second())
                / LatencyInMilliseconds::from(sample_rate.0),
        )
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LatencyMsInvalidity {
    OutOfRange,
}

impl Validate for LatencyMs {
    type Invalidity = LatencyMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !self.0.is_finite() || *self < Self::min(),
                LatencyMsInvalidity::OutOfRange,
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
pub struct LoudnessLufs(pub LufsValue);

// Loudness is measured according to ITU-R BS.1770 in "Loudness Units
// relative to Full Scale" (LUFS) with 1 LU = 1 dB.
// EBU R128 proposes a target level of -23 LUFS while the ReplayGain v2
// specification (RG2) proposes -18 LUFS for achieving similar perceptive
// results compared to ReplayGain v1 (RG1).
impl LoudnessLufs {
    pub const fn unit_of_measure() -> &'static str {
        "LUFS"
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LoudnessLufsInvalidity {
    OutOfRange,
}

impl Validate for LoudnessLufs {
    type Invalidity = LoudnessLufsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.0.is_finite(), LoudnessLufsInvalidity::OutOfRange)
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
