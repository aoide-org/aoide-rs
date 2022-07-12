// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::{entity::Entity, playlist::PlaylistWithEntriesSummary};

#[cfg(feature = "frontend")]
mod _core {
    pub(super) use aoide_core::playlist::{Entity, Playlist};
}

mod _inner {
    pub(super) use crate::_inner::playlist::EntityWithEntriesSummary;
}

pub type EntityWithEntriesSummary = Entity<PlaylistWithEntriesSummary>;

#[cfg(feature = "backend")]
#[must_use]
pub fn export_entity_with_entries_summary(
    from: _inner::EntityWithEntriesSummary,
) -> EntityWithEntriesSummary {
    let _inner::EntityWithEntriesSummary { entity, entries } = from;
    let (hdr, playlist) = entity.into();
    let body = PlaylistWithEntriesSummary {
        playlist: playlist.into(),
        entries: entries.into(),
    };
    Entity(hdr.into(), body)
}

#[cfg(feature = "frontend")]
pub fn import_entity_with_entries_summary(
    entity_with_entries_summary: EntityWithEntriesSummary,
) -> anyhow::Result<_inner::EntityWithEntriesSummary> {
    let Entity(hdr, body) = entity_with_entries_summary;
    let PlaylistWithEntriesSummary { playlist, entries } = body;
    let playlist: _core::Playlist = playlist.try_into()?;
    let entity = _core::Entity::new(hdr, playlist);
    let entries = entries.into();
    Ok(_inner::EntityWithEntriesSummary { entity, entries })
}
