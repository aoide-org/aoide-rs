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

use crate::music::{
    key::KeySignature,
    time::{TempoBpm, TimeSignature},
};

mod _core {
    pub use aoide_core::{
        music::{
            key::KeySignature,
            time::{TempoBpm, TimeSignature},
        },
        track::music::MusicalSignature,
    };
}

use aoide_core::track::music::MusicalSignatureLocks;

///////////////////////////////////////////////////////////////////////
// MusicalSignature
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MusicalSignature {
    #[serde(rename = "bpm", skip_serializing_if = "Option::is_none")]
    tempo_bpm: Option<TempoBpm>,

    #[serde(rename = "bar", skip_serializing_if = "Option::is_none")]
    time_signature: Option<TimeSignature>,

    #[serde(rename = "key", skip_serializing_if = "Option::is_none")]
    key_signature: Option<KeySignature>,

    #[serde(rename = "lck", skip_serializing_if = "IsDefault::is_default", default)]
    locks: u8,
}

impl From<_core::MusicalSignature> for MusicalSignature {
    fn from(from: _core::MusicalSignature) -> Self {
        let _core::MusicalSignature {
            tempo_bpm,
            time_signature,
            key_signature,
            locks,
        } = from;
        Self {
            tempo_bpm: tempo_bpm.map(Into::into),
            time_signature: time_signature.map(Into::into),
            key_signature: key_signature.map(Into::into),
            locks: locks.bits(),
        }
    }
}

impl From<MusicalSignature> for _core::MusicalSignature {
    fn from(from: MusicalSignature) -> Self {
        let MusicalSignature {
            tempo_bpm,
            time_signature,
            key_signature,
            locks,
        } = from;
        Self {
            tempo_bpm: tempo_bpm.map(Into::into),
            time_signature: time_signature.map(Into::into),
            key_signature: key_signature.map(Into::into),
            locks: MusicalSignatureLocks::from_bits_truncate(locks),
        }
    }
}
