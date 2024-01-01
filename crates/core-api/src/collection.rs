// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, Entity};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadScope {
    Entity,
    EntityWithSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaSourceSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub media_sources: MediaSourceSummary,
    pub playlists: PlaylistSummary,
    pub tracks: TrackSummary,
}

/// Collection with an optional summary
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionWithSummary {
    pub collection: Collection,
    pub summary: Option<Summary>,
}

impl CollectionWithSummary {
    #[must_use]
    pub const fn without_summary(collection: Collection) -> Self {
        Self {
            collection,
            summary: None,
        }
    }
}

impl From<Collection> for CollectionWithSummary {
    fn from(from: Collection) -> Self {
        Self::without_summary(from)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EntityWithSummary {
    pub entity: Entity,
    pub summary: Option<Summary>,
}
