// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    media::content::ContentPath,
    track::{Entity, Track},
};

#[derive(Clone, Debug, Default)]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<ContentPath<'static>>,
    pub skipped: Vec<ContentPath<'static>>,
    pub failed: Vec<ContentPath<'static>>,
    pub not_imported: Vec<ContentPath<'static>>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}
