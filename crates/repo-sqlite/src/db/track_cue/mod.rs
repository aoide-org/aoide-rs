// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::cue::Cue;
use aoide_repo::track::RecordId;

pub(crate) mod models;
pub(crate) mod schema;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub cue: Cue,
}
