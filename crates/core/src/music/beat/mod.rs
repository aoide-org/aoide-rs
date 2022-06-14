// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use crate::prelude::*;

use std::{f64, fmt, ops::Not as _};

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

// For counting beats within a measure, phrase, or section
pub type BeatNumber = u16;

pub type BeatsPerMeasure = BeatNumber;

#[must_use]
pub fn is_valid_beats_per_measure(beats_per_measure: BeatsPerMeasure) -> bool {
    beats_per_measure > 0
}

pub type BeatUnit = BeatNumber;

#[must_use]
pub fn is_valid_beat_unit(beat_unit: BeatUnit) -> bool {
    beat_unit > 0
}

/// Musical time signature
///
/// <https://en.wikipedia.org/wiki/Time_signature>
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TimeSignature {
    /// The number of beats in each measure or bar
    ///
    /// This number appears as the nominator/upper value in the stacked notation.
    pub beats_per_measure: BeatsPerMeasure,

    /// The note value that counts as one beat
    ///
    /// This number appears as the denominator/lower value in the stacked notation.
    ///
    /// Example: 4 for a quarter-note
    pub beat_unit: Option<BeatUnit>,
}

fn gcd(nom: BeatNumber, denom: BeatNumber) -> BeatNumber {
    debug_assert!(nom > 0);
    debug_assert!(denom > 0);
    let mut a = nom;
    let mut b = denom;
    loop {
        let c = a % b;
        if c == 0 {
            return b;
        }
        a = b;
        b = c;
    }
}

impl TimeSignature {
    #[must_use]
    pub fn new(beats_per_measure: BeatsPerMeasure, beat_unit: Option<BeatUnit>) -> Self {
        Self {
            beats_per_measure,
            beat_unit,
        }
    }

    #[allow(clippy::absurd_extreme_comparisons)]
    #[must_use]
    pub fn primary_beat_count(self) -> BeatNumber {
        let Self {
            beats_per_measure,
            beat_unit,
        } = self;
        if beats_per_measure <= 0 {
            return 0;
        }
        if let Some(beat_unit) = beat_unit {
            if beat_unit <= 0 {
                return beats_per_measure;
            }
            beats_per_measure / gcd(beats_per_measure, beat_unit)
        } else {
            beats_per_measure
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TimeSignatureInvalidity {
    BeatsPerMeasure,
    BeatUnit,
}

impl Validate for TimeSignature {
    type Invalidity = TimeSignatureInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                !is_valid_beats_per_measure(self.beats_per_measure),
                Self::Invalidity::BeatsPerMeasure,
            )
            .invalidate_if(
                self.beat_unit
                    .map(|beat_unit| is_valid_beat_unit(beat_unit).not())
                    .unwrap_or_default(),
                Self::Invalidity::BeatUnit,
            )
            .into()
    }
}

impl fmt::Display for TimeSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(beat_unit) = self.beat_unit {
            write!(f, "{}/{}", self.beats_per_measure, beat_unit)
        } else {
            self.beats_per_measure.fmt(f)
        }
    }
}

///////////////////////////////////////////////////////////////////////
// MeasurePosition
///////////////////////////////////////////////////////////////////////

/// Total number of measures in a musical score
///
/// Counting starts with the first measure at offset 0.0.
/// ...
/// measure number -2 = measure offset [-2.0, -1.0)
/// measure number -1 = measure offset [-1.0,  0.0)
/// measure number +1 = measure offset [ 0.0,  1.0)
/// measure number +2 = measure offset [ 1.0,  2.0)
/// ...
pub type MeasureOffset = f32;

pub type MeasureNumber = i32;

#[must_use]
pub fn is_valid_measure_number(measure_number: MeasureNumber) -> bool {
    measure_number != 0
}

pub type PreciseMeasureOffset = f64;

/// Beat offset within a measure `[0.0, beat_per_measure)`
pub type BeatOffsetInMeasure = f32;

#[must_use]
pub fn is_valid_beat_offset_in_measure(
    beat_offset_in_measure: BeatOffsetInMeasure,
    beats_per_measure: BeatNumber,
) -> bool {
    beat_offset_in_measure >= 0.0
        && beat_offset_in_measure < BeatOffsetInMeasure::from(beats_per_measure)
}

pub type TotalBeatOffset = f64;

pub type BeatDelta = f64;

/// Musical score/sheet position in measures and beats
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct MeasurePosition {
    /// The (absolute) measure offset
    ///
    /// The offset in measures since the 1st beat in the 1st measure.
    pub measure_offset: MeasureOffset,

    /// The (relative) beat offset within the current measure
    ///
    /// The offset in beats since the start of the current measure.
    ///
    /// The minimum value 0.0 marks the 1st beat or *downbeat* in the
    /// current measure. The maximum value must be strictly less than
    /// the value of *beats per measure* for the current time signature.
    pub beat_offset_in_measure: BeatOffsetInMeasure,
}

impl MeasurePosition {
    #[must_use]
    pub fn from_measure_number_and_beat_offset(
        measure_number: MeasureNumber,
        beat_offset_in_measure: BeatOffsetInMeasure,
    ) -> Self {
        let measure_offset = if measure_number > 0 {
            (measure_number - 1) as MeasureOffset
        } else {
            // The invalid measure number 0 is consistently mapped to
            // offset 0.0.
            measure_number as MeasureOffset
        };
        Self {
            measure_offset,
            beat_offset_in_measure,
        }
    }

    #[must_use]
    pub fn from_total_beat_offset(
        total_beat_offset: TotalBeatOffset,
        beats_per_measure: BeatNumber,
    ) -> Self {
        let beats_per_measure = TotalBeatOffset::from(beats_per_measure);
        Self {
            measure_offset: total_beat_offset.div_euclid(beats_per_measure) as MeasureOffset,
            beat_offset_in_measure: total_beat_offset.rem_euclid(beats_per_measure)
                as BeatOffsetInMeasure,
        }
    }

    #[must_use]
    pub fn is_valid_in_measure(self, beats_per_measure: BeatNumber) -> bool {
        debug_assert!(beats_per_measure > 0);
        self.beat_offset_in_measure < BeatOffsetInMeasure::from(beats_per_measure)
    }

    #[must_use]
    pub fn precise_measure_offset(self, beats_per_measure: BeatNumber) -> PreciseMeasureOffset {
        debug_assert!(self.is_valid());
        debug_assert!(self.is_valid_in_measure(beats_per_measure));
        let Self {
            measure_offset,
            beat_offset_in_measure,
        } = self;
        let fractional_measure = PreciseMeasureOffset::from(beat_offset_in_measure)
            / PreciseMeasureOffset::from(beats_per_measure);
        PreciseMeasureOffset::from(measure_offset.floor()) + fractional_measure
    }

    #[must_use]
    pub fn measure_number(self) -> MeasureNumber {
        let Self {
            measure_offset,
            beat_offset_in_measure: _,
        } = self;
        let measure_number = measure_offset.floor().clamp(
            MeasureNumber::min_value() as MeasureOffset,
            MeasureNumber::max_value() as MeasureOffset,
        ) as MeasureNumber;
        if measure_number < 0 {
            measure_number
        } else {
            // Measure number 0 is invalid
            measure_number + 1
        }
    }

    #[must_use]
    pub fn total_beat_offset(self, beats_per_measure: BeatNumber) -> TotalBeatOffset {
        debug_assert!(self.is_valid());
        debug_assert!(self.is_valid_in_measure(beats_per_measure));
        let Self {
            measure_offset,
            beat_offset_in_measure,
        } = self;
        PreciseMeasureOffset::from(measure_offset.floor())
            * PreciseMeasureOffset::from(beats_per_measure)
            + PreciseMeasureOffset::from(beat_offset_in_measure)
    }

    #[must_use]
    pub fn move_by_beats(self, beats_per_measure: BeatNumber, beat_delta: BeatDelta) -> Self {
        debug_assert!(self.is_valid());
        debug_assert!(self.is_valid_in_measure(beats_per_measure));
        let Self {
            measure_offset,
            beat_offset_in_measure,
        } = self;
        let beats_per_measure = BeatDelta::from(beats_per_measure);
        let beat_offset = BeatDelta::from(beat_offset_in_measure) + beat_delta;
        let measure_offset = measure_offset.floor()
            + beat_offset.div_euclid(beats_per_measure) as BeatOffsetInMeasure;
        let beat_offset_in_measure = beat_offset.rem_euclid(beats_per_measure) as MeasureOffset;
        Self {
            measure_offset,
            beat_offset_in_measure,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MeasurePositionInvalidity {
    BeatOffset,
}

impl Validate for MeasurePosition {
    type Invalidity = MeasurePositionInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(
                self.beat_offset_in_measure < 0.0,
                Self::Invalidity::BeatOffset,
            )
            .into()
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
