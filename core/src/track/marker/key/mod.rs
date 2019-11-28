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

use crate::{
    audio::{PositionMs, PositionMsInvalidity},
    music::key::*,
};

use std::f64;

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub state: State,

    pub source: Option<String>,

    pub start: PositionMs,

    pub end: Option<PositionMs>,

    pub key: KeySignature,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerInvalidity {
    Start(PositionMsInvalidity),
    End(PositionMsInvalidity),
    ReverseDirection,
    Key(KeySignatureInvalidity),
}

impl Validate for Marker {
    type Invalidity = MarkerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        let mut context = ValidationContext::new();
        if let Some(end) = self.end {
            context = context
                .validate_with(&end, MarkerInvalidity::End)
                .invalidate_if(self.start > end, MarkerInvalidity::ReverseDirection);
        }
        context
            .validate_with(&self.start, MarkerInvalidity::Start)
            .validate_with(&self.key, MarkerInvalidity::Key)
            .into()
    }
}

#[derive(Debug)]
pub struct Markers;

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
    Ranges,
}

impl Markers {
    pub fn uniform_key(markers: &[Marker]) -> Option<KeySignature> {
        let mut key = None;
        for marker in markers {
            if marker.key.is_valid() {
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

    pub fn validate<'a>(
        markers: impl Iterator<Item = &'a Marker>,
    ) -> ValidationResult<MarkersInvalidity> {
        let mut min_pos = PositionMs(f64::NEG_INFINITY);
        let mut ranges_violation = false;
        markers
            .fold(ValidationContext::new(), |context, marker| {
                if min_pos > marker.start {
                    ranges_violation = true;
                }
                min_pos = marker.start;
                context.validate_with(marker, MarkersInvalidity::Marker)
            })
            .invalidate_if(ranges_violation, MarkersInvalidity::Ranges)
            .into()
    }
}
