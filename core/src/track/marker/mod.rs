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

pub mod beat;
pub mod key;
pub mod position;

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

#[derive(Clone, Debug, PartialEq)]
pub struct Markers {
    pub positions: Vec<position::Marker>,
    pub beats: Vec<beat::Marker>,
    pub keys: Vec<key::Marker>,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Positions(position::MarkersInvalidity),
    Beats(beat::MarkersInvalidity),
    Keys(key::MarkersInvalidity),
}

impl Validate for Markers {
    type Invalidity = MarkersInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .merge_result_with(
                position::Markers::validate(self.positions.iter()),
                MarkersInvalidity::Positions,
            )
            .merge_result_with(
                beat::Markers::validate(self.beats.iter()),
                MarkersInvalidity::Beats,
            )
            .merge_result_with(
                key::Markers::validate(self.keys.iter()),
                MarkersInvalidity::Keys,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
