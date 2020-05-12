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

use crate::audio::{
    sample::{SamplePosition, SamplePositionInvalidity},
    PositionMs, PositionMsInvalidity,
};

pub mod beat;
pub mod cue;
pub mod key;

pub type Number = i32;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum State {
    ReadWrite,
    ReadOnly,
}

impl State {
    pub const fn default() -> Self {
        State::ReadWrite
    }
}

impl Default for State {
    fn default() -> Self {
        State::default()
    }
}

pub type FrameOffset = i64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Position {
    /// The offset from the start of the track in milliseconds
    pub millis: PositionMs,

    /// The offset from the start of the track in sample frames
    ///
    /// The samples are counted separately for each channel!
    pub samples: Option<SamplePosition>,
}

impl From<PositionMs> for Position {
    fn from(millis: PositionMs) -> Self {
        Self {
            millis,
            samples: None,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum PositionInvalidity {
    PositionMs(PositionMsInvalidity),
    SamplePosition(SamplePositionInvalidity),
}

impl Validate for Position {
    type Invalidity = PositionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.millis, PositionInvalidity::PositionMs)
            .validate_with(&self.samples, PositionInvalidity::SamplePosition)
            .into()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Markers {
    pub beats: beat::Markers,
    pub cues: cue::Markers,
    pub keys: key::Markers,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Beats(beat::MarkersInvalidity),
    Cues(cue::MarkersInvalidity),
    Keys(key::MarkersInvalidity),
}

impl Validate for Markers {
    type Invalidity = MarkersInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.beats, MarkersInvalidity::Beats)
            .validate_with(&self.cues, MarkersInvalidity::Cues)
            .validate_with(&self.keys, MarkersInvalidity::Keys)
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
