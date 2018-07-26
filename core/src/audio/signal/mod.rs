// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use audio::sample::*;
use audio::*;

use std::fmt;
use std::u32;

pub type BitsPerSample = u8;

///////////////////////////////////////////////////////////////////////
/// BitRate
///////////////////////////////////////////////////////////////////////

pub type BitsPerSecond = u32;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct BitRateBps(BitsPerSecond);

impl BitRateBps {
    pub const UNIT_OF_MEASURE: &'static str = "bps";

    pub const MIN: Self = BitRateBps(1);
    pub const MAX: Self = BitRateBps(u32::MAX);

    pub fn from_bps(bps: BitsPerSecond) -> Self {
        BitRateBps(bps)
    }

    pub fn bps(self) -> BitsPerSecond {
        self.0
    }

    pub fn is_valid(&self) -> bool {
        *self >= Self::MIN
    }
}

impl fmt::Display for BitRateBps {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.bps(), Self::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// SampleRate
///////////////////////////////////////////////////////////////////////

pub type SamplesPerSecond = u32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SampleRateHz(SamplesPerSecond);

impl SampleRateHz {
    pub const UNIT_OF_MEASURE: &'static str = "Hz";

    pub const MIN: Self = SampleRateHz(1);
    pub const MAX: Self = SampleRateHz(u32::MAX);

    pub const COMPACT_DISC: Self = SampleRateHz(44_100);
    pub const STUDIO_48KHZ: Self = SampleRateHz(48_000);
    pub const STUDIO_96KHZ: Self = SampleRateHz(96_000);
    pub const STUDIO_192KHZ: Self = SampleRateHz(192_000);

    pub fn from_hz(hz: SamplesPerSecond) -> Self {
        SampleRateHz(hz)
    }

    pub fn hz(self) -> SamplesPerSecond {
        self.0
    }

    pub fn is_valid(&self) -> bool {
        *self >= Self::MIN
    }
}

impl fmt::Display for SampleRateHz {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.hz(), SampleRateHz::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// PcmSignal
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PcmSignal {
    pub channel_layout: ChannelLayout,

    pub sample_layout: SampleLayout,

    pub sample_rate: SampleRateHz,
}

impl PcmSignal {
    pub fn is_valid(&self) -> bool {
        self.sample_rate.is_valid()
    }

    pub fn bitrate(&self, bits_per_sample: BitsPerSample) -> Option<BitRateBps> {
        if self.is_valid() {
            let bps = BitsPerSecond::from(*self.channel_layout.channel_count())
                * self.sample_rate.hz()
                * BitsPerSecond::from(bits_per_sample);
            Some(BitRateBps::from_bps(bps))
        } else {
            None
        }
    }
}

///////////////////////////////////////////////////////////////////////
/// Latency
///////////////////////////////////////////////////////////////////////

pub type LatencyInMilliseconds = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct LatencyMs(LatencyInMilliseconds);

impl LatencyMs {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    const UNITS_PER_SECOND: LatencyInMilliseconds = 1_000 as LatencyInMilliseconds;

    pub fn from_ms(ms: LatencyInMilliseconds) -> Self {
        LatencyMs(ms)
    }

    pub fn from_sample_duration_and_rate(
        sample_length: SampleLength,
        sample_rate: SampleRateHz,
    ) -> LatencyMs {
        Self::from_ms(
            (*sample_length * Self::UNITS_PER_SECOND)
                / LatencyInMilliseconds::from(sample_rate.hz()),
        )
    }

    pub fn ms(self) -> LatencyInMilliseconds {
        self.0
    }
}

impl fmt::Display for LatencyMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.ms(), LatencyMs::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// Loudness
///////////////////////////////////////////////////////////////////////

pub type LufsValue = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Lufs(LufsValue);

// Loudness is measured in "Loudness Units relative to Full Scale" (LUFS) with 1 LU = 1 dB.
impl Lufs {
    pub const UNIT_OF_MEASURE: &'static str = "LUFS";

    pub fn new(value: LufsValue) -> Self {
        Lufs(value)
    }
}

impl Deref for Lufs {
    type Target = LufsValue;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub enum Loudness {
    // Loudness measured according to ITU-R BS.1770 in LUFS.
    // EBU R128 proposes a target level of -23 LUFS while the
    // ReplayGain v2 specification (RG2) proposes -18 LUFS for
    // achieving similar perceptive results compared to
    // ReplayGain v1 (RG1).
    #[serde(rename = "itu-bs1770-lufs")]
    ItuBs1770(Lufs),
}

impl Loudness {
    pub fn is_valid(&self) -> bool {
        true
    }
}

impl fmt::Display for Loudness {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Loudness::ItuBs1770(lufs) => {
                write!(f, "ITU-R BS.1770 {} {}", *lufs, Lufs::UNIT_OF_MEASURE)
            }
        }
    }
}
