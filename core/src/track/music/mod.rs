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

use super::*;

use crate::music::{
    key::{KeySignature, KeySignatureInvalidity},
    time::{TempoBpm, TempoBpmInvalidity, TimeSignature, TimeSignatureInvalidity},
};

use bitflags::bitflags;

bitflags! {
    pub struct MusicalSignatureLocks: u8 {
        const TEMPO_BPM_LOCKED      = 0b00000001;
        const TIME_SIGNATURE_LOCKED = 0b00000010;
        const KEY_SIGNATURE_LOCKED  = 0b00000100;
    }
}

impl MusicalSignatureLocks {
    pub fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for MusicalSignatureLocks {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct MusicalSignatureLocksInvalidity;

impl Validate for MusicalSignatureLocks {
    type Invalidity = MusicalSignatureLocksInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .invalidate_if(!self.is_valid(), MusicalSignatureLocksInvalidity)
            .into()
    }
}

/// Properties that define the musical signature of a track.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MusicalSignature {
    /// The nominal or main musical speed of the track
    pub tempo_bpm: Option<TempoBpm>,

    /// The nominal or main musical time signature of the track
    pub time_signature: Option<TimeSignature>,

    /// The nominal or main musical key signature of the track
    ///
    /// For tracks with varying keys often only the initial key
    /// is mentioned and stored in file tags.
    pub key_signature: Option<KeySignature>,

    pub locks: MusicalSignatureLocks,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MusicalSignatureInvalidity {
    TempoBpm(TempoBpmInvalidity),
    TimeSignature(TimeSignatureInvalidity),
    KeySignature(KeySignatureInvalidity),
    Locks(MusicalSignatureLocksInvalidity),
}

impl Validate for MusicalSignature {
    type Invalidity = MusicalSignatureInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.tempo_bpm, MusicalSignatureInvalidity::TempoBpm)
            .validate_with(
                &self.time_signature,
                MusicalSignatureInvalidity::TimeSignature,
            )
            .validate_with(
                &self.key_signature,
                MusicalSignatureInvalidity::KeySignature,
            )
            .validate_with(&self.locks, MusicalSignatureInvalidity::Locks)
            .into()
    }
}
