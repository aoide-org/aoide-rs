// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use bitflags::bitflags;

use crate::{
    music::{
        beat::{TimeSignature, TimeSignatureInvalidity},
        key::KeySignature,
        tempo::{TempoBpm, TempoBpmInvalidity},
    },
    prelude::*,
};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct MetricsFlags: u8 {
        const TEMPO_BPM_LOCKED      = 0b0000_0001;
        const KEY_SIGNATURE_LOCKED  = 0b0000_0010;
        const TIME_SIGNATURE_LOCKED = 0b0000_0100;

        /// Non-fractional integer BPM
        ///
        /// Some file tags only store integer values. On re-import such an imprecise
        /// value should be ignored to prevent overwriting the more precise internal
        /// value that is stored in the database.
        ///
        /// Imprecise integer BPM values are only re-imported if this flag is set.
        const TEMPO_BPM_INTEGER     = 0b0001_0000;
    }
}

impl MetricsFlags {
    #[must_use]
    pub const fn is_valid(self) -> bool {
        Self::all().contains(self)
    }
}

impl Default for MetricsFlags {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
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
    pub key_signature: Option<KeySignature>,

    /// The nominal or main musical time signature of the track
    pub time_signature: Option<TimeSignature>,

    pub flags: MetricsFlags,
}

#[derive(Copy, Clone, Debug)]
pub enum MetricsInvalidity {
    TempoBpm(TempoBpmInvalidity),
    TimeSignature(TimeSignatureInvalidity),
    Flags(MetricsFlagsInvalidity),
}

impl Validate for Metrics {
    type Invalidity = MetricsInvalidity;

    fn validate(&self) -> ValidationResult<Self::Invalidity> {
        ValidationContext::new()
            .validate_with(&self.tempo_bpm, Self::Invalidity::TempoBpm)
            .validate_with(&self.time_signature, Self::Invalidity::TimeSignature)
            .validate_with(&self.flags, Self::Invalidity::Flags)
            .into()
    }
}
