// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod models;
pub(crate) mod schema;

use crate::{db::track::Scope, prelude::*};

use aoide_core::track::title::*;

use aoide_repo::track::RecordId;

#[derive(Debug)]
pub struct Record {
    pub track_id: RecordId,
    pub scope: Scope,
    pub title: Title,
}
