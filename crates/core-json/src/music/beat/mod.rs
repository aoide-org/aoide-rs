// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::prelude::*;

mod _core {
    pub(super) use aoide_core::music::beat::*;
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
pub enum TimeSignature {
    Top(_core::BeatNumber),
    TopBottom(_core::BeatNumber, _core::BeatNumber),
}

impl From<TimeSignature> for _core::TimeSignature {
    fn from(from: TimeSignature) -> Self {
        use TimeSignature::*;
        match from {
            Top(beats_per_measure) => _core::TimeSignature {
                beats_per_measure,
                beat_unit: None,
            },
            TopBottom(beats_per_measure, beat_unit) => _core::TimeSignature {
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
            TimeSignature::TopBottom(beats_per_measure, beat_unit)
        } else {
            TimeSignature::Top(beats_per_measure)
        }
    }
}

#[cfg(test)]
mod tests;
