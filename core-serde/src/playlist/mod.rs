// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub mod track;

use super::*;

use crate::util::color::Color;

use aoide_core::util::clock::{TickInstant, TickType, Ticks};

mod _core {
    pub use aoide_core::{
        entity::EntityHeader,
        playlist::{track, *},
    };
}

///////////////////////////////////////////////////////////////////////
// PlaylistItem
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub enum PlaylistItem {
    #[serde(rename = "sep")]
    Separator,

    #[serde(rename = "trk")]
    Track(track::Item),
    //
    // TODO: Add other kinds of playlist items
    //#[serde(rename = "x")]
    //Transition(PlaylistTransition),
}

impl From<PlaylistItem> for _core::PlaylistItem {
    fn from(from: PlaylistItem) -> Self {
        use PlaylistItem::*;
        match from {
            Separator => Self::Separator,
            Track(item) => Self::Track(item.into()),
        }
    }
}

impl From<_core::PlaylistItem> for PlaylistItem {
    fn from(from: _core::PlaylistItem) -> Self {
        use _core::PlaylistItem::*;
        match from {
            Separator => Self::Separator,
            Track(item) => Self::Track(item.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// PlaylistEntry
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistEntry {
    #[serde(rename = "add")]
    added_at: TickType,

    #[serde(rename = "itm")]
    item: PlaylistItem,
}

impl From<PlaylistEntry> for _core::PlaylistEntry {
    fn from(from: PlaylistEntry) -> Self {
        let PlaylistEntry { item, added_at } = from;
        Self {
            item: item.into(),
            added_at: TickInstant(Ticks(added_at)),
        }
    }
}

impl From<_core::PlaylistEntry> for PlaylistEntry {
    fn from(from: _core::PlaylistEntry) -> Self {
        let _core::PlaylistEntry { item, added_at } = from;
        Self {
            item: item.into(),
            added_at: (added_at.0).0,
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Playlist
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct Playlist {
    #[serde(rename = "nam")]
    name: String,

    #[serde(rename = "dsc", skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(rename = "typ", skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    color: Option<Color>,

    #[serde(rename = "lst")]
    entries: Vec<PlaylistEntry>,
}

impl From<Playlist> for _core::Playlist {
    fn from(from: Playlist) -> Self {
        let Playlist {
            name,
            description,
            r#type,
            color,
            entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<_core::Playlist> for Playlist {
    fn from(from: _core::Playlist) -> Self {
        let _core::Playlist {
            name,
            description,
            r#type,
            color,
            entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

pub type Entity = crate::entity::Entity<Playlist>;

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

///////////////////////////////////////////////////////////////////////
// PlaylistBriefEntries
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistBriefEntries {
    #[serde(rename = "cnt")]
    count: usize,

    #[serde(rename = "add", skip_serializing_if = "Option::is_none")]
    added_minmax: Option<(TickType, TickType)>,

    #[serde(rename = "trk")]
    tracks: PlaylistBriefTracks,
}

impl From<_core::PlaylistBriefEntries> for PlaylistBriefEntries {
    fn from(from: _core::PlaylistBriefEntries) -> Self {
        let _core::PlaylistBriefEntries {
            count,
            added_minmax,
            tracks,
        } = from;
        Self {
            count,
            added_minmax: added_minmax.map(|(min, max)| ((min.0).0, (max.0).0)),
            tracks: tracks.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistBriefTracks {
    #[serde(rename = "cnt")]
    count: usize,
}

impl From<_core::PlaylistBriefTracks> for PlaylistBriefTracks {
    fn from(from: _core::PlaylistBriefTracks) -> Self {
        let _core::PlaylistBriefTracks { count } = from;
        Self { count }
    }
}

///////////////////////////////////////////////////////////////////////
// PlaylistBrief
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistBrief {
    #[serde(rename = "nam")]
    name: String,

    #[serde(rename = "dsc", skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(rename = "typ", skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(rename = "col", skip_serializing_if = "Option::is_none")]
    color: Option<Color>,

    #[serde(rename = "lst")]
    entries: PlaylistBriefEntries,
}

impl From<_core::Playlist> for PlaylistBrief {
    fn from(from: _core::Playlist) -> Self {
        let entries = from.entries_brief().into();
        let _core::Playlist {
            name,
            description,
            r#type,
            color,
            entries: _entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            entries,
        }
    }
}

impl From<_core::PlaylistBrief> for PlaylistBrief {
    fn from(from: _core::PlaylistBrief) -> Self {
        let _core::PlaylistBrief {
            name,
            description,
            r#type,
            color,
            entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            entries: entries.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// BriefEntity
///////////////////////////////////////////////////////////////////////

pub type BriefEntity = crate::entity::Entity<PlaylistBrief>;

impl From<_core::Entity> for BriefEntity {
    fn from(from: _core::Entity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
