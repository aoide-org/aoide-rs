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

use crate::music::{key::*, time::*};

// Count the total number of measures in a track or musical score
pub type MeasureNumber = u16;

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub position: Position,

    /// The current tempo valid from this position onwards
    pub tempo: Option<TempoBpm>,

    /// The current time signature valid from this position onwards
    ///
    /// The time signature is valid until the next marker or until
    /// the end of a track.
    pub time_signature: Option<TimeSignature>,

    /// The current key signature valid from this position onwards
    ///
    /// The key signature is valid until the next marker or until
    /// the end of a track.
    pub key_signature: Option<KeySignature>,

    /// The beat number within the current measure at this position
    ///
    /// Beat number 1 marks a *downbeat*. The maximum valid number is
    /// `time_signature.beats_per_measure` for the current time signature
    /// and marks an *upbeat* for the subsequent measure.
    ///
    /// If this fields is missing all subsequent beat numbers are
    /// calculated using the current tempo and time signature starting
    /// from the last known beat number.
    pub beat_number: Option<BeatNumber>,

    /// The measure number since the start of the track (or musical score)
    /// up to this position
    ///
    /// The first identifiable measure of the track starts with number 1.
    ///
    /// If this fields is missing all subsequent measure numbers are calculated
    /// using the current tempo and time signature starting from the last
    /// known measure and beat number.
    pub measure_number: Option<MeasureNumber>,
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerInvalidity {
    Position(PositionInvalidity),
    ReverseDirection,
    Tempo(TempoBpmInvalidity),
    TimeSignature(TimeSignatureInvalidity),
    KeySignature(KeySignatureInvalidity),
    BeatNumber,
    MeasureNumber,
    MissingFields,
}

impl Validate for Marker {
    type Invalidity = MarkerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.position, MarkerInvalidity::Position)
            .validate_with(&self.tempo, MarkerInvalidity::Tempo)
            .validate_with(&self.time_signature, MarkerInvalidity::TimeSignature)
            .validate_with(&self.key_signature, MarkerInvalidity::KeySignature)
            .invalidate_if(
                self.tempo.is_none()
                    && self.time_signature.is_none()
                    && self.key_signature.is_none()
                    && self.measure_number.is_none()
                    && self.beat_number.is_none(),
                MarkerInvalidity::MissingFields,
            )
            .invalidate_if(
                self.time_signature
                    .and_then(|t| {
                        self.beat_number
                            .map(|beat_number| beat_number < 1 || beat_number > t.beats_per_measure)
                    })
                    .unwrap_or_default(),
                MarkerInvalidity::BeatNumber,
            )
            .invalidate_if(
                self.measure_number
                    .map(|measure_number| measure_number < 1)
                    .unwrap_or_default(),
                MarkerInvalidity::MeasureNumber,
            )
            .into()
    }
}

fn uniform_tempo_from_markers<'a>(markers: impl Iterator<Item = &'a Marker>) -> Option<TempoBpm> {
    let mut with_tempo = markers.filter_map(|m| m.tempo);
    if let Some(tempo) = with_tempo.next() {
        for t in with_tempo {
            if t != tempo {
                return None;
            }
        }
        return Some(tempo);
    }
    None
}

fn uniform_time_signature_from_markers<'a>(
    markers: impl Iterator<Item = &'a Marker>,
) -> Option<TimeSignature> {
    let mut with_time_signature = markers.filter_map(|marker| marker.time_signature);
    if let Some(time_signature) = with_time_signature.next() {
        for t in with_time_signature {
            if t != time_signature {
                return None;
            }
        }
        return Some(time_signature);
    }
    None
}

fn uniform_key_signature_from_markers<'a>(
    markers: impl Iterator<Item = &'a Marker>,
) -> Option<KeySignature> {
    let mut with_key_signature = markers.filter_map(|marker| marker.key_signature);
    if let Some(key_signature) = with_key_signature.next() {
        for k in with_key_signature {
            if k != key_signature {
                return None;
            }
        }
        return Some(key_signature);
    }
    None
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Markers {
    pub state: State,
    pub markers: Vec<Marker>,
}

impl Markers {
    pub fn uniform_tempo(&self) -> Option<TempoBpm> {
        uniform_tempo_from_markers(self.markers.iter())
    }

    pub fn uniform_time_signature(&self) -> Option<TimeSignature> {
        uniform_time_signature_from_markers(self.markers.iter())
    }

    pub fn uniform_key_signature(&self) -> Option<KeySignature> {
        uniform_key_signature_from_markers(self.markers.iter())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkersInvalidity {
    Marker(MarkerInvalidity),
}

fn validate_markers<'a>(
    markers: impl Iterator<Item = &'a Marker>,
) -> ValidationResult<MarkersInvalidity> {
    markers
        .fold(ValidationContext::new(), |context, marker| {
            context.validate_with(marker, MarkersInvalidity::Marker)
        })
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
