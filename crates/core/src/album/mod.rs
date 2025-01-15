// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{track::album::Kind, util::clock::DateOrDateTime};

/// Read-only album summary aggregated from multiple [`crate::track::Track`]s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AlbumSummary {
    pub artist: String,

    pub title: String,

    pub track_count: u32,

    pub kind: Option<Kind>,

    pub publisher: Option<String>,

    pub min_recorded_at: Option<DateOrDateTime>,
    pub max_recorded_at: Option<DateOrDateTime>,

    pub min_released_at: Option<DateOrDateTime>,
    pub max_released_at: Option<DateOrDateTime>,

    pub min_released_orig_at: Option<DateOrDateTime>,
    pub max_released_orig_at: Option<DateOrDateTime>,
}
