// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::content::ContentPath,
    track::{Entity, Track},
};

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<ContentPath>,
    pub skipped: Vec<ContentPath>,
    pub failed: Vec<ContentPath>,
    pub not_imported: Vec<ContentPath>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}
