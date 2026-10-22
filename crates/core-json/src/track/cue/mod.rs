// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::cue::{BankIndex, CueFlags, SlotIndex};

use crate::{audio::PositionMs, prelude::*};

mod _core {
    pub(super) use aoide_core::track::cue::{Cue, InMarker, OutMarker, OutMode};
}

///////////////////////////////////////////////////////////////////////
// OutMode
///////////////////////////////////////////////////////////////////////

/// The playback behavior when reaching the out position.
///
/// Possible values:
///
/// * 0 - cont: Continue playback when reaching the out position (default if missing).
/// * 1 - stop: Stop playback when reaching the out position.
/// * 2 - next: Continue playback at the in position of the cue with the next slot index, i.e.
///   slot index + 1. If next slot is empty or if that cue has no in position then playback
///   continues (default behavior).
/// * 3 - loop: Continue playback at the in position when reaching the out position. If no in
///   position is defined then playback continues (default behavior).
#[derive(Debug, Serialize_repr, Deserialize_repr)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema_repr),
    schemars(rename = "CueOutMode")
)]
#[repr(u8)]
pub enum OutMode {
    Cont = 0,
    Stop = 1,
    Next = 2,
    Loop = 3,
}

impl From<_core::OutMode> for OutMode {
    fn from(from: _core::OutMode) -> Self {
        use _core::OutMode as From;
        match from {
            From::Cont => Self::Cont,
            From::Stop => Self::Stop,
            From::Loop => Self::Loop,
            From::Next => Self::Next,
        }
    }
}

impl From<OutMode> for _core::OutMode {
    fn from(from: OutMode) -> Self {
        use OutMode as From;
        match from {
            From::Cont => Self::Cont,
            From::Stop => Self::Stop,
            From::Loop => Self::Loop,
            From::Next => Self::Next,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "CueInMarker")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InMarker {
    pub position_ms: PositionMs,
}

impl From<_core::InMarker> for InMarker {
    fn from(from: _core::InMarker) -> Self {
        let _core::InMarker { position } = from;
        Self {
            position_ms: position,
        }
    }
}

impl From<InMarker> for _core::InMarker {
    fn from(from: InMarker) -> Self {
        let InMarker { position_ms } = from;
        Self {
            position: position_ms,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "CueOutMarker")
)]
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
            position_ms: position,
            mode: mode.map(Into::into),
        }
    }
}

impl From<OutMarker> for _core::OutMarker {
    fn from(from: OutMarker) -> Self {
        let OutMarker { position_ms, mode } = from;
        Self {
            position: position_ms,
            mode: mode.map(Into::into),
        }
    }
}

#[expect(clippy::trivially_copy_pass_by_ref)] // Required for serde
fn is_default_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "TrackCue")
)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Cue {
    /// The bank index.
    ///
    /// Banks are used for distinguishing different kinds of cues.
    pub bank_index: BankIndex,

    /// The slot index within a bank.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot_index: Option<SlotIndex>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_marker: Option<InMarker>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub out_marker: Option<OutMarker>,

    /// Application-defined tag for distinguishing cues within the same `bank`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// A custom, user-defined string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    /// A bitmask with flags, e.g. for locking to prevent unintended changes.
    ///
    /// Bits:
    ///
    /// * 0x01: Locked (read-only)
    /// * 0x02: Activate implicitly, e.g. when loading a track.
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
