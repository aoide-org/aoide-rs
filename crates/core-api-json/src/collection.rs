// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core_json::{collection::Collection, entity::Entity};

use crate::prelude::*;

mod _core {
    pub use aoide_core::collection::{Collection, Entity};
}

mod _inner {
    pub use crate::_inner::collection::{
        EntityWithSummary, MediaSourceSummary, PlaylistSummary, Summary, TrackSummary,
    };
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MediaSourceSummary {
    pub total_count: u64,
}

#[cfg(feature = "frontend")]
impl From<MediaSourceSummary> for _inner::MediaSourceSummary {
    fn from(from: MediaSourceSummary) -> Self {
        let MediaSourceSummary { total_count } = from;
        Self { total_count }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::MediaSourceSummary> for MediaSourceSummary {
    fn from(from: _inner::MediaSourceSummary) -> Self {
        let _inner::MediaSourceSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistSummary {
    pub total_count: u64,
}

#[cfg(feature = "frontend")]
impl From<PlaylistSummary> for _inner::PlaylistSummary {
    fn from(from: PlaylistSummary) -> Self {
        let PlaylistSummary { total_count } = from;
        Self { total_count }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::PlaylistSummary> for PlaylistSummary {
    fn from(from: _inner::PlaylistSummary) -> Self {
        let _inner::PlaylistSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSummary {
    pub total_count: u64,
}

#[cfg(feature = "frontend")]
impl From<TrackSummary> for _inner::TrackSummary {
    fn from(from: TrackSummary) -> Self {
        let TrackSummary { total_count } = from;
        Self { total_count }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::TrackSummary> for TrackSummary {
    fn from(from: _inner::TrackSummary) -> Self {
        let _inner::TrackSummary { total_count } = from;
        Self { total_count }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub media_sources: MediaSourceSummary,
    pub playlists: PlaylistSummary,
    pub tracks: TrackSummary,
}

#[cfg(feature = "frontend")]
impl From<Summary> for _inner::Summary {
    fn from(from: Summary) -> Self {
        let Summary {
            media_sources,
            playlists,
            tracks,
        } = from;
        Self {
            media_sources: media_sources.into(),
            playlists: playlists.into(),
            tracks: tracks.into(),
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Summary> for Summary {
    fn from(from: _inner::Summary) -> Self {
        let _inner::Summary {
            media_sources,
            playlists,
            tracks,
        } = from;
        Self {
            media_sources: media_sources.into(),
            playlists: playlists.into(),
            tracks: tracks.into(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionWithSummary {
    #[serde(flatten)]
    pub collection: Collection,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<Summary>,
}

pub type EntityWithSummary = Entity<CollectionWithSummary>;

#[cfg(feature = "backend")]
#[must_use]
pub fn export_entity_with_summary(from: _inner::EntityWithSummary) -> EntityWithSummary {
    let _inner::EntityWithSummary { entity, summary } = from;
    let (hdr, collection) = entity.into();
    let body = CollectionWithSummary {
        collection: collection.into(),
        summary: summary.map(Into::into),
    };
    Entity(hdr.into(), body)
}

#[cfg(feature = "frontend")]
pub fn import_entity_with_summary(
    entity_with_summary: EntityWithSummary,
) -> anyhow::Result<(_core::Entity, Option<_inner::Summary>)> {
    let Entity(hdr, body) = entity_with_summary;
    let CollectionWithSummary {
        collection,
        summary,
    } = body;
    let collection: _core::Collection = collection.try_into()?;
    let entity = _core::Entity::new(hdr, collection);
    let summary = summary.map(Into::into);
    Ok((entity, summary))
}
