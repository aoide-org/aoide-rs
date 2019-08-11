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

use crate::{audio::PositionMs, music::key::*, util::IsDefault};

use std::f64;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyMarker {
    pub start: PositionMs,

    pub end: Option<PositionMs>,

    pub key: KeySignature,
}

impl KeyMarker {
    pub fn uniform_key(markers: &[KeyMarker]) -> Option<KeySignature> {
        let mut key = None;
        for marker in markers {
            if !marker.key.is_default() {
                if let Some(key) = key {
                    if marker.key != key {
                        return None;
                    }
                }
                key = Some(marker.key);
            }
        }
        key
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KeyMarkerValidation {
    Start,
    End,
    Direction,
    Key,
}

impl Validate<KeyMarkerValidation> for KeyMarker {
    fn validate(&self) -> ValidationResult<KeyMarkerValidation> {
        let mut errors = ValidationErrors::default();
        errors.map_and_merge_result(self.start.validate(), |()| KeyMarkerValidation::Start);
        if let Some(end) = self.end {
            errors.map_and_merge_result(end.validate(), |()| KeyMarkerValidation::End);
            if self.start > end {
                errors.add_error(KeyMarkerValidation::Direction, Violation::Invalid);
            }
        }
        errors.map_and_merge_result(self.key.validate(), |()| KeyMarkerValidation::Key);
        errors.into_result()
    }
}

#[derive(Debug)]
pub struct KeyMarkers;

#[derive(Clone, Copy, Debug)]
pub enum KeyMarkersValidation {
    Marker(KeyMarkerValidation),
    OrderedAndNonOverlappingRanges,
}

impl KeyMarkers {
    pub fn validate(markers: &[KeyMarker]) -> ValidationResult<KeyMarkersValidation> {
        let mut errors = ValidationErrors::default();
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        for marker in markers {
            errors.map_and_merge_result(marker.validate(), KeyMarkersValidation::Marker);
            if min_pos > marker.start {
                errors.add_error(
                    KeyMarkersValidation::OrderedAndNonOverlappingRanges,
                    Violation::Invalid,
                );
            }
            min_pos = marker.start;
        }
        errors.into_result()
    }
}
