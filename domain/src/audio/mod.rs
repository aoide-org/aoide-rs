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

use std::{fmt, time::Duration};

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

pub type PositionInMilliseconds = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PositionMs(pub PositionInMilliseconds);

impl PositionMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }
}

impl Validate for PositionMs {
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

impl fmt::Display for PositionMs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:+} {}", self.0, Self::unit_of_measure())
    }
}

///////////////////////////////////////////////////////////////////////
// Duration
///////////////////////////////////////////////////////////////////////

pub type DurationInMilliseconds = f64;

#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DurationMs(pub DurationInMilliseconds);

impl DurationMs {
    pub const fn unit_of_measure() -> &'static str {
        "ms"
    }

    pub const fn empty() -> Self {
        Self(0f64)
    }
}

impl Validate for DurationMs {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();
        if !(self.0.is_finite() && *self >= Self::empty()) {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioEncoder {
    #[serde(rename = "n", skip_serializing_if = "String::is_empty", default)]
    #[validate(length(min = 1))]
    pub name: String,

    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    pub settings: Option<String>,
}

///////////////////////////////////////////////////////////////////////
// AudioContent
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AudioContent {
    #[serde(rename = "ch", skip_serializing_if = "IsDefault::is_default", default)]
    #[validate]
    pub channels: Channels,

    #[serde(rename = "ms", skip_serializing_if = "IsDefault::is_default", default)]
    #[validate]
    pub duration: DurationMs,

    #[serde(rename = "hz", skip_serializing_if = "IsDefault::is_default", default)]
    #[validate]
    pub sample_rate: SampleRateHz,

    #[serde(rename = "bps", skip_serializing_if = "IsDefault::is_default", default)]
    #[validate]
    pub bit_rate: BitRateBps,

    #[serde(rename = "lufs", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub loudness: Option<LoudnessLufs>,

    #[serde(rename = "enc", skip_serializing_if = "Option::is_none")]
    #[validate]
    pub encoder: Option<AudioEncoder>,
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
