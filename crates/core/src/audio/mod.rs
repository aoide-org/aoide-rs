// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::prelude::*;

pub mod channel;
pub mod sample;
pub mod signal;

use self::{channel::*, sample::*, signal::*};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum PositionMsInvalidity {
    OutOfRange,
}

impl Validate for PositionMs {
    type Invalidity = PositionMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.0.is_finite(), Self::Invalidity::OutOfRange)
            .into()
    }
}

impl fmt::Display for PositionMs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:+} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationInMilliseconds = f64;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct DurationMs(DurationInMilliseconds);

impl DurationMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    pub const fn from_inner(inner: DurationInMilliseconds) -> Self {
        Self(inner)
    }

    pub const fn to_inner(self) -> DurationInMilliseconds {
        let Self(inner) = self;
        inner
    }

    pub const fn empty() -> Self {
        Self(0_f64)
    }

    pub fn is_empty(self) -> bool {
        self <= Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DurationMsInvalidity {
    OutOfRange,
}

impl Validate for DurationMs {
    type Invalidity = DurationMsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !(self.0.is_finite() && *self >= Self::empty()),
                Self::Invalidity::OutOfRange,
            )
            .into()
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.to_inner(), Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AudioContent {
    pub duration: Option<DurationMs>,

    pub channels: Option<Channels>,

    pub sample_rate: Option<SampleRateHz>,

    pub bitrate: Option<BitrateBps>,

    pub loudness: Option<LoudnessLufs>,

    // Encoder and settings
    pub encoder: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AudioContentInvalidity {
    Duration(DurationMsInvalidity),
    Channels(ChannelsInvalidity),
    SampleRate(SampleRateHzInvalidity),
    Bitrate(BitrateBpsInvalidity),
    Loudness(LoudnessLufsInvalidity),
    EncoderEmpty,
}

impl Validate for AudioContent {
    type Invalidity = AudioContentInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.duration, Self::Invalidity::Duration)
            .validate_with(&self.channels, Self::Invalidity::Channels)
            .validate_with(&self.sample_rate, Self::Invalidity::SampleRate)
            .validate_with(&self.bitrate, Self::Invalidity::Bitrate)
            .validate_with(&self.loudness, Self::Invalidity::Loudness)
            .invalidate_if(
                self.encoder
                    .as_deref()
                    .map(str::trim)
                    .map(str::is_empty)
                    .unwrap_or(false),
                Self::Invalidity::EncoderEmpty,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
