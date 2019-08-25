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

pub mod channel;
pub mod sample;
pub mod signal;

use self::{channel::*, sample::*, signal::*};

use crate::util::IsEmpty;

use std::{fmt, time::Duration};

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

pub type PositionInMilliseconds = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct PositionMs(pub PositionInMilliseconds);

impl PositionMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PositionMsValidation {
    OutOfRange,
}

impl Validate for PositionMs {
    type Validation = PositionMsValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(!self.0.is_finite(), PositionMsValidation::OutOfRange);
        context.into_result()
    }
}

impl fmt::Display for PositionMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:+} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationInMilliseconds = f64;

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct DurationMs(pub DurationInMilliseconds);

impl DurationMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    pub const fn empty() -> Self {
        Self(0f64)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DurationMsValidation {
    OutOfRange,
}

impl Validate for DurationMs {
    type Validation = DurationMsValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            !(self.0.is_finite() && *self >= Self::empty()),
            DurationMsValidation::OutOfRange,
        );
        context.into_result()
    }
}

impl IsEmpty for DurationMs {
    fn is_empty(&self) -> bool {
        *self <= Self::empty()
    }
}

impl From<Duration> for DurationMs {
    fn from(duration: Duration) -> Self {
        let secs = duration.as_secs() as DurationInMilliseconds;
        let subsec_nanos = DurationInMilliseconds::from(duration.subsec_nanos());
        Self(
            secs * DurationInMilliseconds::from(1_000)
                + subsec_nanos / DurationInMilliseconds::from(1_000_000),
        )
    }
}

impl fmt::Display for DurationMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// AudioEncoder
///////////////////////////////////////////////////////////////////////
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioEncoder {
    pub name: String,

    pub settings: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioEncoderValidation {
    NameEmpty,
}

impl Validate for AudioEncoder {
    type Validation = AudioEncoderValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.add_violation_if(
            self.name.trim().is_empty(),
            AudioEncoderValidation::NameEmpty,
        );
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioContent {
    pub channels: Channels,

    pub duration: DurationMs,

    pub sample_rate: SampleRateHz,

    pub bit_rate: BitRateBps,

    pub loudness: Option<LoudnessLufs>,

    pub encoder: Option<AudioEncoder>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AudioContentValidation {
    Channels(ChannelsValidation),
    Duration(DurationMsValidation),
    SampleRate(SampleRateHzValidation),
    BitRate(BitRateBpsValidation),
    Loudness(LoudnessLufsValidation),
    Encoder(AudioEncoderValidation),
}

impl Validate for AudioContent {
    type Validation = AudioContentValidation;

    fn validate(&self) -> ValidationResult<Self::Validation> {
        let mut context = ValidationContext::default();
        context.map_and_merge_result(self.channels.validate(), AudioContentValidation::Channels);
        context.map_and_merge_result(self.duration.validate(), AudioContentValidation::Duration);
        context.map_and_merge_result(
            self.sample_rate.validate(),
            AudioContentValidation::SampleRate,
        );
        context.map_and_merge_result(self.bit_rate.validate(), AudioContentValidation::BitRate);
        if let Some(loudness) = self.loudness {
            context.map_and_merge_result(loudness.validate(), AudioContentValidation::Loudness);
        }
        if let Some(ref encoder) = self.encoder {
            context.map_and_merge_result(encoder.validate(), AudioContentValidation::Encoder);
        }
        context.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
