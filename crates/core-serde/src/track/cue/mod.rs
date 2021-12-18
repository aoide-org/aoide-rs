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

use crate::{audio::PositionMs, prelude::*};

mod _core {
    pub use aoide_core::{
        audio::{PositionInMilliseconds, PositionMs},
        track::cue::{Cue, OutMode},
    };
}

use aoide_core::{
    track::cue::{BankIndex, CueFlags, InMarker, OutMarker, SlotIndex},
    util::IsDefault,
};

///////////////////////////////////////////////////////////////////////
// OutMode
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr, JsonSchema)]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cue {
    pub bank_index: BankIndex,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_index: Option<SlotIndex>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_position_ms: Option<PositionMs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_position_ms: Option<PositionMs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_mode: Option<OutMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    flags: u8,
}

impl From<_core::Cue> for Cue {
    fn from(from: _core::Cue) -> Self {
        let _core::Cue {
            bank_index,
            slot_index,
            in_marker,
            out_marker,
            label,
            color,
            flags,
        } = from;
        let in_position = in_marker.map(|InMarker { position }| position);
        let (out_position, out_mode) = out_marker
            .map(|OutMarker { position, mode }| (Some(position), mode))
            .unwrap_or((None, None));
        Self {
            bank_index,
            slot_index,
            in_position_ms: in_position.map(Into::into),
            out_position_ms: out_position.map(Into::into),
            out_mode: out_mode.map(Into::into),
            label: label.map(Into::into),
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
            in_position_ms,
            out_position_ms,
            out_mode,
            label,
            color,
            flags,
        } = from;
        let in_marker = in_position_ms.map(|position_ms| InMarker {
            position: position_ms.into(),
        });
        let out_marker = out_position_ms.map(|position_ms| OutMarker {
            position: position_ms.into(),
            mode: out_mode.map(Into::into),
        });
        Self {
            bank_index,
            slot_index,
            in_marker,
            out_marker,
            label: label.map(Into::into),
            color: color.map(Into::into),
            flags: CueFlags::from_bits_truncate(flags),
        }
    }
}
