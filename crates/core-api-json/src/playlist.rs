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

use aoide_core_json::{entity::Entity, playlist::PlaylistWithEntriesSummary};

mod _core {
    pub use aoide_core::playlist::{Entity, EntriesSummary, Playlist};
}

mod _inner {
    pub use crate::_inner::playlist::EntityWithEntriesSummary;
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
