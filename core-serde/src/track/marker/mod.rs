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

#[cfg(feature = "experimental-bnk-markers")]
pub mod bnk;

pub mod cue;

mod _core {
    pub use aoide_core::track::marker::{Markers, Position, State};
}

use aoide_core::util::IsDefault;

use crate::audio::{sample::SamplePosition, PositionMs};

///////////////////////////////////////////////////////////////////////
// Position
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Position {
    Millis(PositionMs),
    MillisSamples(PositionMs, SamplePosition),
}

impl From<Position> for _core::Position {
    fn from(from: Position) -> Self {
        use Position::*;
        match from {
            Millis(millis) => _core::Position {
                millis: millis.into(),
                samples: None,
            },
            MillisSamples(millis, samples) => _core::Position {
                millis: millis.into(),
                samples: Some(samples.into()),
            },
        }
    }
}

impl From<_core::Position> for Position {
    fn from(from: _core::Position) -> Self {
        let _core::Position { millis, samples } = from;
        if let Some(samples) = samples {
            Position::MillisSamples(millis.into(), samples.into())
        } else {
            Position::Millis(millis.into())
        }
    }
}

///////////////////////////////////////////////////////////////////////
// State
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum State {
    ReadWrite = 0,
    ReadOnly = 1,
}

impl Default for State {
    fn default() -> Self {
        _core::State::default().into()
    }
}

impl From<_core::State> for State {
    fn from(from: _core::State) -> Self {
        use _core::State::*;
        match from {
            ReadWrite => State::ReadWrite,
            ReadOnly => State::ReadOnly,
        }
    }
}

impl From<State> for _core::State {
    fn from(from: State) -> Self {
        use _core::State::*;
        match from {
            State::ReadWrite => ReadWrite,
            State::ReadOnly => ReadOnly,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Markers
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Markers {
    #[cfg(feature = "experimental-bnk-markers")]
    #[serde(rename = "bnk", skip_serializing_if = "IsDefault::is_default", default)]
    pub beats_and_keys: bnk::Markers,

    #[serde(rename = "cue", skip_serializing_if = "IsDefault::is_default", default)]
    pub cues: cue::Markers,
}

impl From<_core::Markers> for Markers {
    #[cfg(feature = "experimental-bnk-markers")]
    fn from(from: _core::Markers) -> Self {
        let _core::Markers {
            beats_and_keys,
            cues,
        } = from;
        Self {
            beats_and_keys: beats_and_keys.into(),
            cues: cues.into(),
        }
    }

    #[cfg(not(feature = "experimental-bnk-markers"))]
    fn from(from: _core::Markers) -> Self {
        let _core::Markers { cues } = from;
        Self { cues: cues.into() }
    }
}

impl From<Markers> for _core::Markers {
    #[cfg(feature = "experimental-bnk-markers")]
    fn from(from: Markers) -> Self {
        let Markers {
            beats_and_keys,
            cues,
        } = from;
        Self {
            beats_and_keys: beats_and_keys.into(),
            cues: cues.into(),
        }
    }

    #[cfg(not(feature = "experimental-bnk-markers"))]
    fn from(from: Markers) -> Self {
        let Markers { cues } = from;
        Self { cues: cues.into() }
    }
}
