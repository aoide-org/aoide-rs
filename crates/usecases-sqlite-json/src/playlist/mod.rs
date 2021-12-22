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

use aoide_core::entity::EntityUid;

use aoide_repo::{
    playlist::RecordHeader,
    prelude::{RecordCollector, ReservableRecordCollector},
};

use aoide_core_json::{
    entity::Entity as GenericEntity,
    playlist::{
        Entity, EntityWithEntriesSummary, EntriesSummary, Entry, Playlist,
        PlaylistWithEntriesSummary,
    },
};

use super::*;

mod _core {
    pub use aoide_core::{
        entity::EntityHeader,
        playlist::{Entity, EntriesSummary},
    };
}

pub mod create_collected;
pub mod list_collected;
pub mod patch_entries;
pub mod purge;
pub mod update;

#[derive(Debug, Clone, Default)]
pub struct EntityWithEntriesSummaryCollector(Vec<EntityWithEntriesSummary>);

impl EntityWithEntriesSummaryCollector {
    pub const fn new(inner: Vec<EntityWithEntriesSummary>) -> Self {
        Self(inner)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let inner = Vec::with_capacity(capacity);
        Self(inner)
    }
}

impl From<EntityWithEntriesSummaryCollector> for Vec<EntityWithEntriesSummary> {
    fn from(from: EntityWithEntriesSummaryCollector) -> Self {
        let EntityWithEntriesSummaryCollector(inner) = from;
        inner
    }
}

impl RecordCollector for EntityWithEntriesSummaryCollector {
    type Header = RecordHeader;
    type Record = (_core::Entity, _core::EntriesSummary);

    fn collect(
        &mut self,
        _header: RecordHeader,
        (entity, entries): (_core::Entity, _core::EntriesSummary),
    ) {
        let Self(inner) = self;
        inner.push(merge_entity_with_entries_summary(
            entity.into(),
            entries.into(),
        ));
    }
}

impl ReservableRecordCollector for EntityWithEntriesSummaryCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}

fn merge_entity_with_entries_summary(
    entity: Entity,
    entries: EntriesSummary,
) -> EntityWithEntriesSummary {
    let GenericEntity(hdr, body) = entity;
    let body = PlaylistWithEntriesSummary {
        playlist: body,
        entries,
    };
    GenericEntity(hdr, body)
}
