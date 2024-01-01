// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(untagged, deny_unknown_fields)]
pub enum TimeSignature {
    Top(_core::BeatNumber),
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
