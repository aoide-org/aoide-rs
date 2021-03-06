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

use std::cmp::Ordering;

use crate::{audio::PositionMs, prelude::*};

use num_derive::{FromPrimitive, ToPrimitive};

pub type BankIndex = i16;

pub type SlotIndex = i16;

/// Defines how playback behaves when reaching the out position
/// when active.
///
/// If no behavior is specified then playback continues at the
/// out position.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ToPrimitive, FromPrimitive)]
pub enum OutMode {
    /// Continue playback when reaching the out position.
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

impl Default for OutMode {
    fn default() -> Self {
        OutMode::Cont
    }
}

use bitflags::bitflags;

bitflags! {
    pub struct CueFlags: u8 {
        const LOCKED = 0b00000001;
    }
}

impl CueFlags {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for CueFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
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
pub struct Cue {
    pub bank_index: BankIndex,

    pub slot_index: Option<SlotIndex>,

    pub in_position: Option<PositionMs>,

    pub out_position: Option<PositionMs>,

    pub out_mode: Option<OutMode>,

    pub label: Option<String>,

    pub color: Option<Color>,

    pub flags: CueFlags,
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
        debug_assert!(self.is_canonical())
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CueInvalidity {
    InOrOutPositionMissing,
    LabelEmpty,
    Flags(CueFlagsInvalidity),
}

impl Validate for Cue {
    type Invalidity = CueInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new()
            .invalidate_if(
                self.in_position.is_none() && self.out_position.is_none(),
                CueInvalidity::InOrOutPositionMissing,
            )
            .validate_with(&self.flags, CueInvalidity::Flags);
        if let Some(ref label) = self.label {
            context = context.invalidate_if(label.trim().is_empty(), CueInvalidity::LabelEmpty)
        }
        context.into()
    }
}

#[cfg(test)]
mod tests;
