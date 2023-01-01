// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::playlist::*;

use aoide_repo::{
    playlist::{CollectionRepo as _, EntityRepo as _, RecordHeader},
    prelude::*,
};

use super::*;

pub mod create;
pub mod entries;
pub mod load;
pub mod purge;
pub mod update;
