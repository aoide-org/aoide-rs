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

use crate::music::key::*;

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub position: Position,

    /// The signature at this position
    ///
    /// The signature is valid until the next marker or until
    /// the end of a track.
    pub signature: KeySignature,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerInvalidity {
    Position(PositionInvalidity),
    ReverseDirection,
    KeySignature(KeySignatureInvalidity),
}

impl Validate for Marker {
    type Invalidity = MarkerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.position, MarkerInvalidity::Position)
            .validate_with(&self.signature, MarkerInvalidity::KeySignature)
            .into()
    }
}

fn uniform_key_from_markers<'a>(markers: impl Iterator<Item = &'a Marker>) -> Option<KeySignature> {
    let mut keys = markers.map(|marker| marker.signature);
    if let Some(key) = keys.next() {
        for k in keys {
            if k != key {
                return None;
            }
        }
        return Some(key);
    }
    None
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Markers {
    pub state: State,
    pub markers: Vec<Marker>,
}

impl Markers {
    pub fn uniform_key(&self) -> Option<KeySignature> {
        uniform_key_from_markers(self.markers.iter())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
    Ranges,
}

fn validate_markers<'a>(
    markers: impl Iterator<Item = &'a Marker>,
) -> ValidationResult<MarkersInvalidity> {
    let mut min_pos = PositionMs(f64::NEG_INFINITY);
    let mut ranges_violation = false;
    markers
        .fold(ValidationContext::new(), |context, marker| {
            if min_pos > marker.position.millis {
                ranges_violation = true;
            }
            min_pos = marker.position.millis;
            context.validate_with(marker, MarkersInvalidity::Marker)
        })
        .invalidate_if(ranges_violation, MarkersInvalidity::Ranges)
        .into()
}

impl Validate for Markers {
    type Invalidity = MarkersInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        validate_markers(self.markers.iter())
    }
}

#[cfg(test)]
mod tests;
