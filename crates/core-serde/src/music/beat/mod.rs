// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

mod _core {
    pub use aoide_core::music::beat::*;
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
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
