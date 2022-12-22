// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{playlist::Flags, track::EntityUid as TrackUid};

use crate::{entity::EntityUid, prelude::*, util::clock::DateTime};

mod _core {
    pub(super) use aoide_core::playlist::*;
}

///////////////////////////////////////////////////////////////////////
// Item
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SeparatorItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) kind: Option<String>,
}

impl From<SeparatorItem> for _core::SeparatorItem {
    fn from(from: SeparatorItem) -> Self {
        let SeparatorItem { kind } = from;
        Self { kind }
    }
}

impl From<_core::SeparatorItem> for SeparatorItem {
    fn from(from: _core::SeparatorItem) -> Self {
        let _core::SeparatorItem { kind } = from;
        Self { kind }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TrackItem {
    pub(crate) uid: EntityUid,
}

impl From<TrackItem> for _core::TrackItem {
    fn from(from: TrackItem) -> Self {
        let TrackItem { uid } = from;
        Self {
            uid: TrackUid::from_untyped(uid),
        }
    }
}

impl From<_core::TrackItem> for TrackItem {
    fn from(from: _core::TrackItem) -> Self {
        let _core::TrackItem { uid } = from;
        Self { uid: uid.into() }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub enum Item {
    Separator(SeparatorItem),
    Track(TrackItem),
}

impl From<Item> for _core::Item {
    fn from(from: Item) -> Self {
        match from {
            Item::Separator(item) => Self::Separator(item.into()),
            Item::Track(item) => Self::Track(item.into()),
        }
    }
}

impl From<_core::Item> for Item {
    fn from(from: _core::Item) -> Self {
        use _core::Item::*;
        match from {
            Separator(item) => Self::Separator(item.into()),
            Track(item) => Self::Track(item.into()),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Entry
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
        let (hdr, body) = from.into();
        Self(hdr.into_untyped().into(), body.into())
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
        let (hdr, body) = from.into();
        Self(hdr.into_untyped().into(), body.into())
    }
}

///////////////////////////////////////////////////////////////////////
// EntriesSummary
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

impl From<EntriesSummary> for _core::EntriesSummary {
    fn from(from: EntriesSummary) -> Self {
        let EntriesSummary {
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

impl From<TracksSummary> for _core::TracksSummary {
    fn from(from: TracksSummary) -> Self {
        let TracksSummary { total_count } = from;
        Self { total_count }
    }
}

///////////////////////////////////////////////////////////////////////
// PlaylistWithEntriesSummary
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
