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
    key::KeyCode,
    time::{TempoBpm, TimeSignature},
};

mod _core {
    pub use aoide_core::{
        music::{
            key::KeyCode,
            time::{TempoBpm, TimeSignature},
        },
        track::metric::Metrics,
    };
}

use aoide_core::{music::key::KeySignature, track::metric::MetricsFlags};

///////////////////////////////////////////////////////////////////////
// Metrics
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    tempo_bpm: Option<TempoBpm>,

    #[serde(skip_serializing_if = "Option::is_none")]
    key_code: Option<KeyCode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_signature: Option<TimeSignature>,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    flags: u8,
}

impl From<_core::Metrics> for Metrics {
    fn from(from: _core::Metrics) -> Self {
        let _core::Metrics {
            tempo_bpm,
            key_signature,
            time_signature,
            flags,
        } = from;
        Self {
            tempo_bpm: tempo_bpm.map(Into::into),
            key_code: if key_signature.is_unknown() {
                None
            } else {
                Some(key_signature.code().into())
            },
            time_signature: time_signature.map(Into::into),
            flags: flags.bits(),
        }
    }
}

impl From<Metrics> for _core::Metrics {
    fn from(from: Metrics) -> Self {
        let Metrics {
            tempo_bpm,
            key_code,
            time_signature,
            flags,
        } = from;
        Self {
            tempo_bpm: tempo_bpm.map(Into::into),
            key_signature: key_code
                .map(Into::into)
                .map(KeySignature::new)
                .unwrap_or_else(KeySignature::unknown),
            time_signature: time_signature.map(Into::into),
            flags: MetricsFlags::from_bits_truncate(flags),
        }
    }
}
