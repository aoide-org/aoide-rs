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

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct PositionMs(pub PositionInMilliseconds);

impl PositionMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }
}

impl Validate<()> for PositionMs {
    fn validate(&self) -> ValidationResult<()> {
        let mut errors = ValidationErrors::default();
        if !self.0.is_finite() {
            errors.add_error((), Violation::OutOfBounds);
        }
        errors.into_result()
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

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd)]
pub struct DurationMs(pub DurationInMilliseconds);

impl DurationMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    pub const fn empty() -> Self {
        Self(0f64)
    }
}

impl Validate<()> for DurationMs {
    fn validate(&self) -> ValidationResult<()> {
        let mut errors = ValidationErrors::default();
        if !(self.0.is_finite() && *self >= Self::empty()) {
            errors.add_error((), Violation::OutOfBounds);
        }
        errors.into_result()
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioEncoderValidation {
    Name,
}

const MIN_NAME_LEN: usize = 1;

impl Validate<AudioEncoderValidation> for AudioEncoder {
    fn validate(&self) -> ValidationResult<AudioEncoderValidation> {
        let mut errors = ValidationErrors::default();
        if self.name.len() < MIN_NAME_LEN {
            errors.add_error(
                AudioEncoderValidation::Name,
                Violation::TooShort(validate::Min(MIN_NAME_LEN)),
            );
        }
        errors.into_result()
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioContentValidation {
    Channels,
    Duration,
    SampleRate,
    BitRate,
    Loudness,
    Encoder(AudioEncoderValidation),
}

impl Validate<AudioContentValidation> for AudioContent {
    fn validate(&self) -> ValidationResult<AudioContentValidation> {
        let mut errors = ValidationErrors::default();
        errors.map_and_merge_result(self.channels.validate(), |()| {
            AudioContentValidation::Channels
        });
        errors.map_and_merge_result(self.duration.validate(), |()| {
            AudioContentValidation::Duration
        });
        errors.map_and_merge_result(self.sample_rate.validate(), |()| {
            AudioContentValidation::SampleRate
        });
        errors.map_and_merge_result(self.bit_rate.validate(), |()| {
            AudioContentValidation::BitRate
        });
        if let Some(ref loudness) = self.loudness {
            errors.map_and_merge_result(loudness.validate(), |()| AudioContentValidation::Loudness);
        }
        if let Some(ref encoder) = self.encoder {
            errors.map_and_merge_result(encoder.validate(), AudioContentValidation::Encoder);
        }
        errors.into_result()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
