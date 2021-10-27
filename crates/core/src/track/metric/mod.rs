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

use crate::{
    music::{
        key::KeySignature,
        time::{TempoBpm, TempoBpmInvalidity, TimeSignature, TimeSignatureInvalidity},
    },
    prelude::*,
};

use bitflags::bitflags;

bitflags! {
    pub struct MetricsFlags: u8 {
        const TEMPO_BPM_LOCKED            = 0b0000_0001;
        const KEY_SIGNATURE_LOCKED        = 0b0000_0010;
        const TIME_SIGNATURE_LOCKED       = 0b0000_0100;

        /// Some file tags only store imprecise integer values
        const TEMPO_BPM_NON_FRACTIONAL    = 0b0001_0000;
    }
}

impl MetricsFlags {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for MetricsFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct MetricsFlagsInvalidity;

impl Validate for MetricsFlags {
    type Invalidity = MetricsFlagsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!MetricsFlags::is_valid(*self), MetricsFlagsInvalidity)
            .into()
    }
}

/// Properties that define the musical signature of a track.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metrics {
    /// The nominal or main musical speed of the track
    pub tempo_bpm: Option<TempoBpm>,

    /// The nominal or main musical key signature of the track
    ///
    /// For tracks with varying keys often only the initial key
    /// is mentioned and stored in file tags.
    pub key_signature: KeySignature,

    /// The nominal or main musical time signature of the track
    pub time_signature: Option<TimeSignature>,

    pub flags: MetricsFlags,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MetricsInvalidity {
    TempoBpm(TempoBpmInvalidity),
    TimeSignature(TimeSignatureInvalidity),
    Flags(MetricsFlagsInvalidity),
}

impl Validate for Metrics {
    type Invalidity = MetricsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.tempo_bpm, MetricsInvalidity::TempoBpm)
            .validate_with(&self.time_signature, MetricsInvalidity::TimeSignature)
            .validate_with(&self.flags, MetricsInvalidity::Flags)
            .into()
    }
}
