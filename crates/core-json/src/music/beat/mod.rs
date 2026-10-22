// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::music::beat::*;
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

/// The time signature of a track.
///
/// Either an incomplete signature with only the beats per measure or a complete signature with
/// both beats per measure and beat unit / note value.
///
/// The beats per measure appear as the nominator/upper value in the stacked notation for time
/// signatures and the beat unit, i.e. the note value that counts as one beat, appears as the
/// denominator/lower value, e.g. 4 for a quarter note.
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
pub enum TimeSignature {
    /// Simplified time signature with only the beats per measure and unknown beat unit.
    Top(_core::BeatNumber),
    /// An array with exactly 2 elements: The number of beats per measure followed by the beat unit.
    TopBottom(_core::BeatNumber, _core::BeatNumber),
}

impl From<TimeSignature> for _core::TimeSignature {
    fn from(from: TimeSignature) -> Self {
        use TimeSignature as From;
        match from {
            From::Top(beats_per_measure) => Self {
                beats_per_measure,
                beat_unit: None,
            },
            From::TopBottom(beats_per_measure, beat_unit) => Self {
                beats_per_measure,
                beat_unit: Some(beat_unit),
            },
        }
    }
}

impl From<_core::TimeSignature> for TimeSignature {
    fn from(from: _core::TimeSignature) -> Self {
        let _core::TimeSignature {
            beats_per_measure,
            beat_unit,
        } = from;
        if let Some(beat_unit) = beat_unit {
            Self::TopBottom(beats_per_measure, beat_unit)
        } else {
            Self::Top(beats_per_measure)
        }
    }
}

#[cfg(test)]
mod tests;
