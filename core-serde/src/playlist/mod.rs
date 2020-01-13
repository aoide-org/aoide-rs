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

use aoide_core::util::clock::{TickInstant, TickType, Ticks};

mod _core {
    pub use aoide_core::{entity::EntityHeader, playlist::*};
}

///////////////////////////////////////////////////////////////////////
// PlaylistTrack
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistTrack {
    #[serde(rename = "u")]
    uid: EntityUid,
}

impl From<PlaylistTrack> for _core::PlaylistTrack {
    fn from(from: PlaylistTrack) -> Self {
        let PlaylistTrack { uid } = from;
        Self { uid: uid.into() }
    }
}

impl From<_core::PlaylistTrack> for PlaylistTrack {
    fn from(from: _core::PlaylistTrack) -> Self {
        let _core::PlaylistTrack { uid } = from;
        Self { uid: uid.into() }
    }
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
    Track(PlaylistTrack),
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
            Track(track) => Self::Track(track.into()),
        }
    }
}

impl From<_core::PlaylistItem> for PlaylistItem {
    fn from(from: _core::PlaylistItem) -> Self {
        use _core::PlaylistItem::*;
        match from {
            Separator => Self::Separator,
            Track(track) => Self::Track(track.into()),
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

    #[serde(rename = "c")]
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
#[cfg_attr(test, derive(Eq, PartialEq))]
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
// PlaylistEntriesBrief
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistEntriesBrief {
    #[serde(rename = "m")]
    tracks_count: usize,

    #[serde(rename = "n")]
    entries_count: usize,

    #[serde(rename = "s")]
    entries_since_min: Option<TickType>,

    #[serde(rename = "t")]
    entries_since_max: Option<TickType>,
}

impl From<&_core::Playlist> for PlaylistEntriesBrief {
    fn from(from: &_core::Playlist) -> Self {
        let tracks_count = from.count_tracks();
        let entries_count = from.entries.len();
        let (entries_since_min, entries_since_max) = from
            .entries_since_min_max()
            .map_or((None, None), |(min, max)| (Some(min), Some(max)));
        Self {
            tracks_count,
            entries_count,
            entries_since_min: entries_since_min.map(|min| (min.0).0),
            entries_since_max: entries_since_max.map(|max| (max.0).0),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// PlaylistBrief
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
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

    #[serde(rename = "e")]
    entries_brief: PlaylistEntriesBrief,
}

impl From<_core::Playlist> for PlaylistBrief {
    fn from(from: _core::Playlist) -> Self {
        let entries_brief = (&from).into();
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
            entries_brief,
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
