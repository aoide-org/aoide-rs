// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use aoide_core::track::actor::*;
use aoide_core_api::track::search::Scope;
use aoide_repo::track::RecordId;

use crate::prelude::*;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub scope: Scope,
    pub actor: Actor,
}
