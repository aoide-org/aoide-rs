// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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
pub struct BitRate {
    pub bps: BitsPerSecond,
}

impl BitRate {
    pub const UNIT_OF_MEASURE: &'static str = "bps";

    pub const MIN: Self = BitRate { bps: u32::MIN };

    pub const MAX: Self = BitRate { bps: u32::MAX };

    pub fn bps(bps: BitsPerSecond) -> Self {
        Self { bps }
    }

    pub fn is_valid(&self) -> bool {
        *self > BitRate::default()
    }
}

impl fmt::Display for BitRate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.bps, BitRate::UNIT_OF_MEASURE)
    }
}

///////////////////////////////////////////////////////////////////////
/// Latency
///////////////////////////////////////////////////////////////////////

pub type LatencyInMilliseconds = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Latency {
    pub ms: LatencyInMilliseconds,
}

impl Latency {
    pub const UNIT_OF_MEASURE: &'static str = "ms";

    const UNITS_PER_SECOND: f64 = 1000 as f64;

    pub fn from_sample_duration_and_rate(
        sample_duration: SampleLength,
        sample_rate: SampleRate,
    ) -> Latency {
        let ms = (*sample_duration * Self::UNITS_PER_SECOND) / (sample_rate.hz as f64);
        Latency { ms }
    }
}

impl fmt::Display for Latency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.ms, Latency::UNIT_OF_MEASURE)
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

    pub sample_rate: SampleRate,
}

impl PcmSignal {
    pub fn is_valid(&self) -> bool {
        self.sample_rate.is_valid()
    }

    pub fn bit_rate(&self, bits_per_sample: BitsPerSample) -> Option<BitRate> {
        if self.is_valid() {
            match self.channel_layout.channel_count() {
                channel_count => {
                    let bps = channel_count as BitsPerSecond * self.sample_rate.hz as BitsPerSecond
                        * bits_per_sample as BitsPerSecond;
                    Some(BitRate { bps: bps })
                }
            }
        } else {
            None
        }
    }
}
