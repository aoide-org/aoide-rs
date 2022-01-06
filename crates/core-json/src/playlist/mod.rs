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

use aoide_core::playlist::Flags;

use crate::{prelude::*, util::clock::DateTime};

pub mod track;

mod _core {
    pub use aoide_core::{
        entity::EntityHeader,
        playlist::{track, *},
    };
}

///////////////////////////////////////////////////////////////////////
// Item
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct SeparatorDummy {}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Item {
    Separator(SeparatorDummy),
    Track(track::Item),
}

impl From<Item> for _core::Item {
    fn from(from: Item) -> Self {
        match from {
            Item::Separator(SeparatorDummy {}) => Self::Separator,
            Item::Track(item) => Self::Track(item.into()),
        }
    }
}

impl From<_core::Item> for Item {
    fn from(from: _core::Item) -> Self {
        use _core::Item::*;
        match from {
            Separator => Self::Separator(SeparatorDummy {}),
            Track(item) => Self::Track(item.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Entry
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    added_at: DateTime,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,

    #[serde(flatten)]
    item: Item,
}

impl From<Entry> for _core::Entry {
    fn from(from: Entry) -> Self {
        let Entry {
            added_at,
            title,
            notes,
            item,
        } = from;
        Self {
            added_at: added_at.into(),
            title,
            notes,
            item: item.into(),
        }
    }
}

impl From<_core::Entry> for Entry {
    fn from(from: _core::Entry) -> Self {
        let _core::Entry {
            added_at,
            title,
            notes,
            item,
        } = from;
        Self {
            added_at: added_at.into(),
            title,
            notes,
            item: item.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Playlist
///////////////////////////////////////////////////////////////////////

fn is_default_flags(flags: &u8) -> bool {
    *flags == u8::default()
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Playlist {
    collected_at: DateTime,

    title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,

    #[serde(skip_serializing_if = "is_default_flags", default)]
    flags: u8,
}

impl From<Playlist> for _core::Playlist {
    fn from(from: Playlist) -> Self {
        let Playlist {
            collected_at,
            title,
            kind,
            notes,
            color,
            flags,
        } = from;
        Self {
            collected_at: collected_at.into(),
            title,
            kind,
            notes,
            color: color.map(Into::into),
            flags: Flags::from_bits_truncate(flags),
        }
    }
}

impl From<_core::Playlist> for Playlist {
    fn from(from: _core::Playlist) -> Self {
        let _core::Playlist {
            collected_at,
            title,
            kind,
            notes,
            color,
            flags,
        } = from;
        Self {
            collected_at: collected_at.into(),
            title,
            kind,
            notes,
            color: color.map(Into::into),
            flags: flags.bits(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaylistWithEntries {
    #[serde(flatten)]
    playlist: Playlist,

    entries: Vec<Entry>,
}

impl From<PlaylistWithEntries> for _core::PlaylistWithEntries {
    fn from(from: PlaylistWithEntries) -> Self {
        let PlaylistWithEntries { playlist, entries } = from;
        Self {
            playlist: playlist.into(),
            entries: entries.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<_core::PlaylistWithEntries> for PlaylistWithEntries {
    fn from(from: _core::PlaylistWithEntries) -> Self {
        let _core::PlaylistWithEntries { playlist, entries } = from;
        Self {
            playlist: playlist.into(),
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

pub type EntityWithEntries = crate::entity::Entity<PlaylistWithEntries>;

impl From<EntityWithEntries> for _core::EntityWithEntries {
    fn from(from: EntityWithEntries) -> Self {
        Self::new(from.0, from.1)
    }
}

impl From<_core::EntityWithEntries> for EntityWithEntries {
    fn from(from: _core::EntityWithEntries) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}

///////////////////////////////////////////////////////////////////////
// EntriesSummary
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntriesSummary {
    total_count: usize,

    #[serde(rename = "addedAtMinMax", skip_serializing_if = "Option::is_none")]
    added_at_minmax: Option<(DateTime, DateTime)>,

    tracks: TracksSummary,
}

impl From<_core::EntriesSummary> for EntriesSummary {
    fn from(from: _core::EntriesSummary) -> Self {
        let _core::EntriesSummary {
            total_count,
            added_at_minmax,
            tracks,
        } = from;
        Self {
            total_count,
            added_at_minmax: added_at_minmax.map(|(min, max)| (min.into(), max.into())),
            tracks: tracks.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// TracksSummary
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TracksSummary {
    total_count: usize,
}

impl From<_core::TracksSummary> for TracksSummary {
    fn from(from: _core::TracksSummary) -> Self {
        let _core::TracksSummary { total_count } = from;
        Self { total_count }
    }
}

///////////////////////////////////////////////////////////////////////
// PlaylistWithEntriesSummary
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlaylistWithEntriesSummary {
    #[serde(flatten)]
    pub playlist: Playlist,

    pub entries: EntriesSummary,
}

impl From<_core::PlaylistWithEntriesSummary> for PlaylistWithEntriesSummary {
    fn from(from: _core::PlaylistWithEntriesSummary) -> Self {
        let _core::PlaylistWithEntriesSummary { playlist, entries } = from;
        Self {
            playlist: playlist.into(),
            entries: entries.into(),
        }
    }
}

pub type EntityWithEntriesSummary = crate::entity::Entity<PlaylistWithEntriesSummary>;

impl From<(_core::Entity, _core::EntriesSummary)> for EntityWithEntriesSummary {
    fn from(from: (_core::Entity, _core::EntriesSummary)) -> Self {
        let (entity, entries) = from;
        let body = PlaylistWithEntriesSummary {
            playlist: entity.body.into(),
            entries: entries.into(),
        };
        Self(entity.hdr.into(), body)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
