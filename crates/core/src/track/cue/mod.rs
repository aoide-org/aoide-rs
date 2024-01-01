// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::cmp::Ordering;

use nonicle::{CanonicalOrd, Canonicalize, IsCanonical};
use strum::FromRepr;

use crate::{audio::PositionMs, prelude::*};

pub type BankIndex = i16;

pub type SlotIndex = i16;

/// Defines how playback behaves when reaching the out position
/// when active.
///
/// If no behavior is specified then playback continues at the
/// out position.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, FromRepr)]
#[repr(u8)]
pub enum OutMode {
    /// Continue playback when reaching the out position.
    #[default]
    Cont = 0,

    /// Stop playback when reaching the out position.
    Stop = 1,

    /// Continue playback at the in position of the cue with
    /// the next slot index, i.e. current slot index + 1.
    ///
    /// If the next slot is empty or if that next cue has no in
    /// position then playback continues (default behavior).
    Next = 2,

    /// Continue playback at the in position when reaching
    /// the out position, i.e. repeat and loop.
    ///
    /// If the cue has no in position then playback continues
    /// (default behavior).
    Loop = 3,
}

use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct CueFlags: u8 {
        const LOCKED = 0b0000_0001;
    }
}

impl CueFlags {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for CueFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct CueFlagsInvalidity;

impl Validate for CueFlags {
    type Invalidity = CueFlagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!CueFlags::is_valid(*self), CueFlagsInvalidity)
            .into()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct InMarker {
    pub position: PositionMs,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutMarker {
    pub position: PositionMs,
    pub mode: Option<OutMode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Cue {
    pub bank_index: BankIndex,

    pub slot_index: Option<SlotIndex>,

    pub in_marker: Option<InMarker>,

    pub out_marker: Option<OutMarker>,

    /// Application-defined tag for distinguishing cues
    /// within the same `bank`.
    pub kind: Option<String>,

    /// A custom, user-defined string
    pub label: Option<String>,

    pub color: Option<Color>,

    pub flags: CueFlags,
}

impl Cue {
    #[must_use]
    pub fn is_reverse(&self) -> bool {
        let Self {
            in_marker,
            out_marker,
            ..
        } = self;
        match (in_marker, out_marker) {
            (Some(in_marker), Some(out_marker)) => in_marker.position > out_marker.position,
            _ => false,
        }
    }
}

impl CanonicalOrd for Cue {
    fn canonical_cmp(&self, other: &Self) -> Ordering {
        let Self {
            bank_index: lhs_bank_index,
            slot_index: lhs_slot_index,
            ..
        } = self;
        let Self {
            bank_index: rhs_bank_index,
            slot_index: rhs_slot_index,
            ..
        } = other;
        lhs_bank_index
            .cmp(rhs_bank_index)
            .then(lhs_slot_index.cmp(rhs_slot_index))
    }
}

impl IsCanonical for Cue {
    fn is_canonical(&self) -> bool {
        true
    }
}

impl Canonicalize for Cue {
    fn canonicalize(&mut self) {
        debug_assert!(self.is_canonical());
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CueInvalidity {
    InOrOutMarkerMissing,
    LabelEmpty,
    Flags(CueFlagsInvalidity),
}

impl Validate for Cue {
    type Invalidity = CueInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new()
            .invalidate_if(
                self.in_marker.is_none() && self.out_marker.is_none(),
                Self::Invalidity::InOrOutMarkerMissing,
            )
            .validate_with(&self.flags, Self::Invalidity::Flags);
        if let Some(ref label) = self.label {
            context = context.invalidate_if(label.trim().is_empty(), Self::Invalidity::LabelEmpty);
        }
        context.into()
    }
}

#[cfg(test)]
mod tests;
