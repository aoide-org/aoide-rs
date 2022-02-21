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

use aoide_core::playlist::{Entity, EntriesSummary};

use aoide_repo::{
    playlist::RecordHeader,
    prelude::{RecordCollector, ReservableRecordCollector},
};

#[derive(Debug, Clone)]
pub struct EntityWithEntriesSummary {
    pub entity: Entity,
    pub entries: EntriesSummary,
}

#[derive(Debug, Default)]
pub struct EntityWithEntriesSummaryCollector(Vec<EntityWithEntriesSummary>);

impl EntityWithEntriesSummaryCollector {
    #[must_use]
    pub const fn new(inner: Vec<EntityWithEntriesSummary>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn finish(self) -> Vec<EntityWithEntriesSummary> {
        let Self(inner) = self;
        inner
    }
}

impl RecordCollector for EntityWithEntriesSummaryCollector {
    type Header = RecordHeader;
    type Record = (Entity, EntriesSummary);

    fn collect(&mut self, _header: RecordHeader, (entity, entries): (Entity, EntriesSummary)) {
        let Self(inner) = self;
        inner.push(EntityWithEntriesSummary { entity, entries });
    }
}

impl ReservableRecordCollector for EntityWithEntriesSummaryCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
