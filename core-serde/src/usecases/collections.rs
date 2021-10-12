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

use crate::prelude::*;

mod _core {
    pub use aoide_core::usecases::collections::{
        MediaSourceSummary, PlaylistSummary, Summary, TrackSummary,
    };
}

use crate::{collection::Collection, entity::Entity};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MediaSourceSummary {
    pub total_count: u64,
}

impl From<_core::MediaSourceSummary> for MediaSourceSummary {
    fn from(from: _core::MediaSourceSummary) -> Self {
        let _core::MediaSourceSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistSummary {
    pub total_count: u64,
}

impl From<_core::PlaylistSummary> for PlaylistSummary {
    fn from(from: _core::PlaylistSummary) -> Self {
        let _core::PlaylistSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSummary {
    pub total_count: u64,
}

impl From<_core::TrackSummary> for TrackSummary {
    fn from(from: _core::TrackSummary) -> Self {
        let _core::TrackSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_sources: Option<MediaSourceSummary>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracks: Option<TrackSummary>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlists: Option<PlaylistSummary>,
}

impl From<_core::Summary> for Summary {
    fn from(from: _core::Summary) -> Self {
        let _core::Summary {
            tracks,
            playlists,
            media_sources,
        } = from;
        Self {
            tracks: tracks.map(Into::into),
            playlists: playlists.map(Into::into),
            media_sources: media_sources.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionWithSummary {
    #[serde(flatten)]
    pub collection: Collection,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
}

pub type EntityWithSummary = Entity<CollectionWithSummary>;
