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

mod _core {
    pub use aoide_core::music::time::*;
}

///////////////////////////////////////////////////////////////////////
// Tempo
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TempoBpm(_core::Beats);

impl From<_core::TempoBpm> for TempoBpm {
    fn from(from: _core::TempoBpm) -> Self {
        Self(from.0)
    }
}

impl From<TempoBpm> for _core::TempoBpm {
    fn from(from: TempoBpm) -> Self {
        Self(from.0)
    }
}

///////////////////////////////////////////////////////////////////////
// TimeSignature
///////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TimeSignature {
    Top(_core::BeatNumber),
    TopBottom(_core::BeatNumber, _core::BeatNumber),
}

impl From<TimeSignature> for _core::TimeSignature {
    fn from(from: TimeSignature) -> Self {
        use TimeSignature::*;
        match from {
            Top(top) => _core::TimeSignature { top, bottom: None },
            TopBottom(top, bottom) => _core::TimeSignature {
                top,
                bottom: Some(bottom),
            },
        }
    }
}

impl From<_core::TimeSignature> for TimeSignature {
    fn from(from: _core::TimeSignature) -> Self {
        let _core::TimeSignature { top, bottom } = from;
        if let Some(bottom) = bottom {
            TimeSignature::TopBottom(top, bottom)
        } else {
            TimeSignature::Top(top)
        }
    }
}

#[cfg(test)]
mod tests;
