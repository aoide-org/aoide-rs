// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::track::cue::Cue;
use aoide_repo::track::RecordId;

use crate::prelude::*;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub cue: Cue,
}
