// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::track::cue::{BankIndex, CueFlags, SlotIndex};

use crate::{audio::PositionMs, prelude::*};

mod _core {
    pub use aoide_core::{
        audio::{PositionInMilliseconds, PositionMs},
        track::cue::{Cue, InMarker, OutMarker, OutMode},
    };
}

///////////////////////////////////////////////////////////////////////
// OutMode
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[repr(u8)]
pub enum OutMode {
    Cont = 0,
    Stop = 1,
    Next = 2,
    Loop = 3,
}

impl From<_core::OutMode> for OutMode {
    fn from(from: _core::OutMode) -> Self {
        use _core::OutMode::*;
        match from {
            Cont => OutMode::Cont,
            Stop => OutMode::Stop,
            Loop => OutMode::Loop,
            Next => OutMode::Next,
        }
    }
}

impl From<OutMode> for _core::OutMode {
    fn from(from: OutMode) -> Self {
        use _core::OutMode::*;
        match from {
            OutMode::Cont => Cont,
            OutMode::Stop => Stop,
            OutMode::Loop => Loop,
            OutMode::Next => Next,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InMarker {
    pub position_ms: PositionMs,
}

impl From<_core::InMarker> for InMarker {
    fn from(from: _core::InMarker) -> Self {
        let _core::InMarker { position } = from;
        Self {
            position_ms: position.into(),
        }
    }
}

impl From<InMarker> for _core::InMarker {
    fn from(from: InMarker) -> Self {
        let InMarker { position_ms } = from;
        Self {
            position: position_ms.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutMarker {
    pub position_ms: PositionMs,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<OutMode>,
}

impl From<_core::OutMarker> for OutMarker {
    fn from(from: _core::OutMarker) -> Self {
        let _core::OutMarker { position, mode } = from;
        Self {
            position_ms: position.into(),
            mode: mode.map(Into::into),
        }
    }
}

impl From<OutMarker> for _core::OutMarker {
    fn from(from: OutMarker) -> Self {
        let OutMarker { position_ms, mode } = from;
        Self {
            position: position_ms.into(),
            mode: mode.map(Into::into),
        }
    }
}

fn is_default_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cue {
    pub bank_index: BankIndex,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_index: Option<SlotIndex>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_marker: Option<InMarker>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_marker: Option<OutMarker>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    #[serde(skip_serializing_if = "is_default_flags", default)]
    flags: u8,
}

impl From<_core::Cue> for Cue {
    fn from(from: _core::Cue) -> Self {
        let _core::Cue {
            bank_index,
            slot_index,
            in_marker,
            out_marker,
            kind,
            label,
            color,
            flags,
        } = from;
        Self {
            bank_index,
            slot_index,
            in_marker: in_marker.map(Into::into),
            out_marker: out_marker.map(Into::into),
            kind,
            label,
            color: color.map(Into::into),
            flags: flags.bits(),
        }
    }
}

impl From<Cue> for _core::Cue {
    fn from(from: Cue) -> Self {
        let Cue {
            bank_index,
            slot_index,
            in_marker,
            out_marker,
            kind,
            label,
            color,
            flags,
        } = from;
        Self {
            bank_index,
            slot_index,
            in_marker: in_marker.map(Into::into),
            out_marker: out_marker.map(Into::into),
            kind,
            label,
            color: color.map(Into::into),
            flags: CueFlags::from_bits_truncate(flags),
        }
    }
}
