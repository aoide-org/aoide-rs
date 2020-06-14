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

#[derive(Clone, Debug, PartialEq)]
pub struct Marker {
    pub position: Position,

    /// The current tempo
    ///
    /// This tempo is supposed to be constant and valid from the
    /// current position onwards until the next marker or until
    /// the end of a track.
    pub tempo_bpm: Option<TempoBpm>,

    /// The current time signature
    ///
    /// This time signature is valid from the current position onwards
    /// until the next marker or until the end of a track.
    pub time_signature: Option<TimeSignature>,

    /// The current key signature valid from this position onwards.
    ///
    /// This key signature is valid from the current position onwards
    /// until the next marker or until the end of a track.
    pub key_signature: Option<KeySignature>,

    /// The current position within musical score (measure + beat in measure)
    ///
    /// If this fields is missing it can be extrapolated from a
    /// preceding measure position using the current tempo and time
    /// signature if available.
    pub measure_position: Option<MeasurePosition>,
}

impl Marker {
    pub fn extrapolate_measure_position(
        &self,
        next_position_millis: PositionMs,
    ) -> Option<MeasurePosition> {
        let Self {
            position:
                Position {
                    millis: position_millis,
                    samples: _,
                },
            tempo_bpm,
            time_signature,
            key_signature: _,
            measure_position,
        } = self;
        debug_assert!(*position_millis <= next_position_millis);
        if *position_millis == next_position_millis {
            return *measure_position;
        }
        if let (Some(measure_position), Some(tempo_bpm), Some(beats_per_measure)) = (
            measure_position,
            tempo_bpm,
            time_signature.map(|ts| ts.beats_per_measure),
        ) {
            let delta_millis = next_position_millis.0 - position_millis.0;
            let delta_minutes = delta_millis / 60_000.0;
            let delta_beats = tempo_bpm.0 * delta_minutes;
            Some(measure_position.move_by_beats(beats_per_measure, delta_beats))
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MarkerInvalidity {
    Position(PositionInvalidity),
    ReverseDirection,
    Tempo(TempoBpmInvalidity),
    TimeSignature(TimeSignatureInvalidity),
    KeySignature(KeySignatureInvalidity),
    MeasurePosition(MeasurePositionInvalidity),
    MeasurePositionInvalidBeatsPerMeasure,
    MissingFields,
}

impl Validate for Marker {
    type Invalidity = MarkerInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.position, MarkerInvalidity::Position)
            .validate_with(&self.tempo_bpm, MarkerInvalidity::Tempo)
            .validate_with(&self.time_signature, MarkerInvalidity::TimeSignature)
            .validate_with(&self.key_signature, MarkerInvalidity::KeySignature)
            .validate_with(&self.measure_position, MarkerInvalidity::MeasurePosition)
            .invalidate_if(
                if let (Some(measure_position), Some(beats_per_measure)) = (
                    self.measure_position,
                    self.time_signature.map(|ts| ts.beats_per_measure),
                ) {
                    !measure_position.is_valid_in_measure(beats_per_measure)
                } else {
                    false
                },
                MarkerInvalidity::MeasurePositionInvalidBeatsPerMeasure,
            )
            .invalidate_if(
                self.tempo_bpm.is_none()
                    && self.time_signature.is_none()
                    && self.key_signature.is_none()
                    && self.measure_position.is_none(),
                MarkerInvalidity::MissingFields,
            )
            .into()
    }
}

fn uniform_tempo_from_markers<'a>(markers: impl Iterator<Item = &'a Marker>) -> Option<TempoBpm> {
    let mut with_tempo_bpm = markers.filter_map(|m| m.tempo_bpm);
    if let Some(tempo_bpm) = with_tempo_bpm.next() {
        for t in with_tempo_bpm {
            if t != tempo_bpm {
                return None;
            }
        }
        return Some(tempo_bpm);
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

    /// Reduce and extrapolate the information from all preceding markers
    /// up to the given position.
    pub fn reduce(&self, position_millis: PositionMs) -> Option<Marker> {
        self.markers
            .iter()
            .take_while(|m| m.position.millis <= position_millis)
            .fold(MarkerReducer::new(), |reducer, next_marker| {
                reducer.reduce(next_marker)
            })
            .finish()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MarkerReducer {
    marker: Option<Marker>,
}

impl MarkerReducer {
    pub const fn new() -> Self {
        Self { marker: None }
    }

    pub fn current_marker(&self) -> Option<&Marker> {
        self.marker.as_ref()
    }

    pub fn reduce(mut self, next_marker: &Marker) -> Self {
        if let Some(marker) = &mut self.marker {
            let Marker {
                position: next_position,
                tempo_bpm: next_tempo_bpm,
                time_signature: next_time_signature,
                key_signature: next_key_signature,
                measure_position: next_measure_position,
            } = next_marker;
            debug_assert!(&marker.position <= next_position);
            marker.measure_position = if let Some(next_measure_position) = next_measure_position {
                if next_measure_position.is_valid() {
                    Some(*next_measure_position)
                } else {
                    // Reset
                    None
                }
            } else {
                // Extrapolate the measure position BEFORE updating tempo
                // and time signature!!
                marker.extrapolate_measure_position(next_position.millis)
            };
            if let Some(next_tempo_bpm) = next_tempo_bpm {
                if next_tempo_bpm.is_valid() {
                    marker.tempo_bpm = Some(*next_tempo_bpm);
                } else {
                    // Reset
                    marker.tempo_bpm = None;
                }
            }
            if let Some(next_time_signature) = next_time_signature {
                if next_time_signature.is_valid() {
                    marker.time_signature = Some(*next_time_signature);
                } else {
                    // Reset
                    marker.time_signature = None;
                }
            }
            if let Some(next_key_signature) = next_key_signature {
                if next_key_signature.is_valid() {
                    marker.key_signature = Some(*next_key_signature);
                } else {
                    // Reset
                    marker.key_signature = None;
                }
            }
        } else {
            self.marker = Some(next_marker.clone())
        }
        self
    }

    pub const fn finish(self) -> Option<Marker> {
        let Self { marker } = self;
        marker
    }
}

impl From<MarkerReducer> for Option<Marker> {
    fn from(from: MarkerReducer) -> Self {
        from.finish()
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
