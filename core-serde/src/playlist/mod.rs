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

use super::*;

use crate::{entity::EntityUid, util::color::ColorRgb};

use aoide_core::util::{
    clock::{TickInstant, TickType, Ticks},
    geo::*,
};

mod _core {
    pub use aoide_core::{entity::EntityHeader, playlist::*};
}

///////////////////////////////////////////////////////////////////////
// PlaylistItem
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub enum PlaylistItem {
    #[serde(rename = "s")]
    Separator,

    #[serde(rename = "t")]
    Track(EntityUid),
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
            Track(track_uid) => Self::Track(_core::PlaylistTrack {
                uid: track_uid.into(),
            }),
        }
    }
}

impl From<_core::PlaylistItem> for PlaylistItem {
    fn from(from: _core::PlaylistItem) -> Self {
        use _core::PlaylistItem::*;
        match from {
            Separator => Self::Separator,
            Track(track) => Self::Track(track.uid.into()),
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
    #[serde(rename = "i")]
    item: PlaylistItem,

    #[serde(rename = "s")]
    since: TickType,

    #[serde(rename = "m", skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

impl From<PlaylistEntry> for _core::PlaylistEntry {
    fn from(from: PlaylistEntry) -> Self {
        let PlaylistEntry {
            item,
            since,
            comment,
        } = from;
        Self {
            item: item.into(),
            since: TickInstant(Ticks(since)),
            comment,
        }
    }
}

impl From<_core::PlaylistEntry> for PlaylistEntry {
    fn from(from: _core::PlaylistEntry) -> Self {
        let _core::PlaylistEntry {
            item,
            since,
            comment,
        } = from;
        Self {
            item: item.into(),
            since: (since.0).0,
            comment,
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
    #[serde(rename = "n")]
    name: String,

    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    color: Option<ColorRgb>,

    #[serde(rename = "g", skip_serializing_if = "Option::is_none")]
    location: Option<(GeoCoord, GeoCoord)>,

    #[serde(rename = "e")]
    entries: Vec<PlaylistEntry>,
}

impl From<Playlist> for _core::Playlist {
    fn from(from: Playlist) -> Self {
        let Playlist {
            name,
            description,
            r#type,
            color,
            location,
            entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            location: location.map(|(lat, lon)| GeoPoint { lat, lon }),
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
            location,
            entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            location: location.map(|p| (p.lat, p.lon)),
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
    #[serde(rename = "n")]
    count: usize,

    #[serde(rename = "s", skip_serializing_if = "Option::is_none")]
    since_minmax: Option<(TickType, TickType)>,

    #[serde(rename = "t")]
    tracks: PlaylistBriefTracks,
}

impl From<_core::PlaylistBriefEntries> for PlaylistBriefEntries {
    fn from(from: _core::PlaylistBriefEntries) -> Self {
        let _core::PlaylistBriefEntries {
            count,
            since_minmax,
            tracks,
        } = from;
        Self {
            count,
            since_minmax: since_minmax.map(|(min, max)| ((min.0).0, (max.0).0)),
            tracks: tracks.into(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistBriefTracks {
    #[serde(rename = "n")]
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
    #[serde(rename = "n")]
    name: String,

    #[serde(rename = "d", skip_serializing_if = "Option::is_none")]
    description: Option<String>,

    #[serde(rename = "p", skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,

    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    color: Option<ColorRgb>,

    #[serde(rename = "g", skip_serializing_if = "Option::is_none")]
    location: Option<(GeoCoord, GeoCoord)>,

    #[serde(rename = "e")]
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
            location,
            entries: _entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            location: location.map(|p| (p.lat, p.lon)),
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
            location,
            entries,
        } = from;
        Self {
            name,
            description,
            r#type,
            color: color.map(Into::into),
            location: location.map(|p| (p.lat, p.lon)),
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
