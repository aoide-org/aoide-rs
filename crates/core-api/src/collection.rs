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

impl MediaSourceSummary {
    pub const EMPTY: Self = Self { total_count: 0 };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackSummary {
    pub total_count: u64,
}

impl TrackSummary {
    pub const EMPTY: Self = Self { total_count: 0 };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaylistSummary {
    pub total_count: u64,
}

impl PlaylistSummary {
    pub const EMPTY: Self = Self { total_count: 0 };
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Summary {
    pub media_sources: MediaSourceSummary,
    pub playlists: PlaylistSummary,
    pub tracks: TrackSummary,
}

impl Summary {
    pub const EMPTY: Self = Self {
        media_sources: MediaSourceSummary::EMPTY,
        playlists: PlaylistSummary::EMPTY,
        tracks: TrackSummary::EMPTY,
    };
}

/// Collection with an optional summary
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionWithSummary {
    pub collection: Collection,
    pub summary: Option<Summary>,
}

impl From<Collection> for CollectionWithSummary {
    fn from(collection: Collection) -> Self {
        Self {
            collection,
            summary: None,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EntityWithSummary {
    pub entity: Entity,
    pub summary: Option<Summary>,
}
