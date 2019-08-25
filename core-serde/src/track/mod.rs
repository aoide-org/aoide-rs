// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

pub mod album;
pub mod collection;
pub mod index;
pub mod marker;
pub mod release;
pub mod source;

use self::{album::*, collection::*, index::*, marker::*, release::*, source::*};

use crate::{actor::*, tag::*, title::*};

mod _core {
    pub use aoide_core::track::*;
}

use aoide_core::util::IsEmpty;

///////////////////////////////////////////////////////////////////////
// Track
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Track {
    #[serde(rename = "c", skip_serializing_if = "Vec::is_empty", default)]
    pub collections: Vec<Collection>,

    #[serde(rename = "s", skip_serializing_if = "Vec::is_empty", default)]
    pub media_sources: Vec<MediaSource>,

    #[serde(rename = "r", skip_serializing_if = "Option::is_none")]
    pub release: Option<Release>,

    #[serde(rename = "a", skip_serializing_if = "Option::is_none")]
    pub album: Option<Album>,

    #[serde(rename = "t", skip_serializing_if = "Vec::is_empty", default)]
    pub titles: Vec<Title>,

    #[serde(rename = "p", skip_serializing_if = "Vec::is_empty", default)]
    pub actors: Vec<Actor>,

    #[serde(rename = "i", skip_serializing_if = "IsEmpty::is_empty", default)]
    pub indexes: Indexes,

    #[serde(rename = "m", skip_serializing_if = "IsEmpty::is_empty", default)]
    pub markers: Markers,

    #[serde(rename = "x", skip_serializing_if = "IsEmpty::is_empty", default)]
    pub tags: Tags,
}

impl From<_core::Track> for Track {
    fn from(from: _core::Track) -> Self {
        Self {
            collections: from.collections.into_iter().map(Into::into).collect(),
            media_sources: from.media_sources.into_iter().map(Into::into).collect(),
            release: from.release.map(Into::into),
            album: from.album.map(Into::into),
            titles: from.titles.into_iter().map(Into::into).collect(),
            actors: from.actors.into_iter().map(Into::into).collect(),
            indexes: from.indexes.into(),
            markers: from.markers.into(),
            tags: Tags::encode(from.tags),
        }
    }
}

impl From<Track> for _core::Track {
    fn from(from: Track) -> Self {
        Self {
            collections: from.collections.into_iter().map(Into::into).collect(),
            media_sources: from.media_sources.into_iter().map(Into::into).collect(),
            release: from.release.map(Into::into),
            album: from.album.map(Into::into),
            titles: from.titles.into_iter().map(Into::into).collect(),
            actors: from.actors.into_iter().map(Into::into).collect(),
            indexes: from.indexes.into(),
            markers: from.markers.into(),
            tags: from.tags.decode(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

pub type Entity = crate::entity::Entity<Track>;

impl From<Entity> for _core::Entity {
    fn from(from: Entity) -> Self {
        Self::new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}
