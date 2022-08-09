// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::*;

use crate::music::{beat::TimeSignature, key::KeyCode, tempo::TempoBpm};

mod _core {
    pub(super) use aoide_core::track::metric::Metrics;
}

use aoide_core::{music::key::KeySignature, track::metric::MetricsFlags};

///////////////////////////////////////////////////////////////////////
// Metrics
///////////////////////////////////////////////////////////////////////

fn is_default_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Metrics {
    #[serde(skip_serializing_if = "Option::is_none")]
    tempo_bpm: Option<TempoBpm>,

    #[serde(skip_serializing_if = "Option::is_none")]
    key_code: Option<KeyCode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    time_signature: Option<TimeSignature>,

    #[serde(skip_serializing_if = "is_default_flags", default)]
    flags: u8,
}

impl Metrics {
    pub(crate) fn is_default(&self) -> bool {
        let Self {
            flags,
            key_code,
            tempo_bpm,
            time_signature,
        } = self;
        is_default_flags(flags)
            && key_code.is_none()
            && tempo_bpm.is_none()
            && time_signature.is_none()
    }
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

#[cfg(test)]
mod tests;
