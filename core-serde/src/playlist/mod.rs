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
// PlaylistEntry
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(deny_unknown_fields)]
pub struct PlaylistEntry {
    #[serde(rename = "u")]
    track_uid: EntityUid,

    #[serde(rename = "t")]
    since: TickType,

    #[serde(rename = "c")]
    comment: Option<String>,
}

impl From<PlaylistEntry> for _core::PlaylistEntry {
    fn from(from: PlaylistEntry) -> Self {
        let PlaylistEntry {
            track_uid,
            since,
            comment,
        } = from;
        Self {
            track_uid: track_uid.into(),
            since: TickInstant(Ticks(since)),
            comment,
        }
    }
}

impl From<_core::PlaylistEntry> for PlaylistEntry {
    fn from(from: _core::PlaylistEntry) -> Self {
        let _core::PlaylistEntry {
            track_uid,
            since,
            comment,
        } = from;
        Self {
            track_uid: track_uid.into(),
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

    #[serde(rename = "t", skip_serializing_if = "Option::is_none")]
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
