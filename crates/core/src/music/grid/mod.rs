// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{
    beat::{MeasurePosition, TimeSignature},
    key::KeySignature,
    tempo::TempoBpm,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TempoAnchor {
    pub tempo_bpm: TempoBpm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyAnchor {
    pub key_signature: KeySignature,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BeatAnchor {
    pub time_signature: TimeSignature,
    pub measure_position: Option<MeasurePosition>,
}
