// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use std::{fmt, u32};

///////////////////////////////////////////////////////////////////////

pub type BitsPerSample = u8;

///////////////////////////////////////////////////////////////////////
// BitRate
///////////////////////////////////////////////////////////////////////

pub type BitsPerSecond = u32;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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

impl Validate for BitRateBps {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !(*self >= Self::min() && *self <= Self::max()) {
            errors.add(
                Self::unit_of_measure(),
                ValidationError::new("invalid value"),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl fmt::Display for BitRateBps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SamplesPerSecond = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
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

impl Validate for SampleRateHz {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !(*self >= Self::min() && *self <= Self::max()) {
            errors.add(
                Self::unit_of_measure(),
                ValidationError::new("invalid value"),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl fmt::Display for SampleRateHz {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// PcmSignal
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PcmSignal {
    pub channel_layout: ChannelLayout,

    pub sample_layout: SampleLayout,

    #[validate]
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

///////////////////////////////////////////////////////////////////////
// Latency
///////////////////////////////////////////////////////////////////////

pub type LatencyInMilliseconds = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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

impl Validate for LatencyMs {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !self.0.is_finite() || *self < Self::min() {
            errors.add(
                Self::unit_of_measure(),
                ValidationError::new("invalid value"),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl fmt::Display for LatencyMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Loudness
///////////////////////////////////////////////////////////////////////

pub type LufsValue = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
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

impl Validate for LoudnessLufs {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !self.0.is_finite() {
            errors.add(
                Self::unit_of_measure(),
                ValidationError::new("invalid value"),
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl fmt::Display for LoudnessLufs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
