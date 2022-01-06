// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aoide_core::collection::Collection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MediaSourceSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlaylistSummary {
    pub total_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Summary {
    pub media_sources: MediaSourceSummary,
    pub playlists: PlaylistSummary,
    pub tracks: TrackSummary,
}

/// Collection with an optional summary
#[derive(Clone, Debug, Eq, PartialEq)]
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
