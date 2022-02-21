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

use aoide_core::track::Entity;
use aoide_repo::{
    prelude::{RecordCollector, ReservableRecordCollector},
    track::RecordHeader,
};

#[derive(Debug, Default)]
pub struct EntityCollector(Vec<Entity>);

impl EntityCollector {
    #[must_use]
    pub const fn new(inner: Vec<Entity>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn finish(self) -> Vec<Entity> {
        let Self(inner) = self;
        inner
    }
}

impl From<EntityCollector> for Vec<Entity> {
    fn from(from: EntityCollector) -> Self {
        let EntityCollector(inner) = from;
        inner
    }
}

impl RecordCollector for EntityCollector {
    type Header = RecordHeader;
    type Record = Entity;

    fn collect(&mut self, _record_header: RecordHeader, entity: Entity) {
        let Self(inner) = self;
        inner.push(entity);
    }
}

impl ReservableRecordCollector for EntityCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
