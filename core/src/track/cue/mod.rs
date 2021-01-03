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
    /// Stop playback when reaching the out position.
    Stop = 0,

    /// Continue playback at the in position of the cue with
    /// the next slot index, i.e. current slot index + 1.
    ///
    /// If the next slot is empty or if that next cue has no in
    /// position then playback continues (default behavior).
    Next = 1,

    /// Continue playback at the in position when reaching
    /// the out positon, i.e. repeat and loop.
    ///
    /// If the cue has no in position then playback continues
    /// (default behavior).
    Loop = 2,
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
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CueInvalidity {
    InOrOutPositionMissing,
    LabelEmpty,
}

impl Validate for Cue {
    type Invalidity = CueInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new().invalidate_if(
            self.in_position.is_none() && self.out_position.is_none(),
            CueInvalidity::InOrOutPositionMissing,
        );
        if let Some(ref label) = self.label {
            context = context.invalidate_if(label.trim().is_empty(), CueInvalidity::LabelEmpty)
        }
        context.into()
    }
}
