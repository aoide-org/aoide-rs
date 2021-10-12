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

use super::*;

mod _core {
    pub use aoide_core::{
        collection::Entity, entity::EntityHeader, usecases::collections::Summary,
    };
}

use aoide_core::entity::EntityUid;

use aoide_repo::{
    collection::RecordHeader,
    prelude::{RecordCollector, ReservableRecordCollector},
};

use aoide_core_serde::{
    collection::{Collection, Entity},
    entity::Entity as GenericEntity,
    usecases::collections::{CollectionWithSummary, EntityWithSummary, Summary},
};

///////////////////////////////////////////////////////////////////////

pub mod create;
pub mod delete;
pub mod load_all;
pub mod load_one;
pub mod update;

#[derive(Debug, Clone, Default)]
pub struct EntityCollector(Vec<EntityWithSummary>);

impl EntityCollector {
    pub const fn new(inner: Vec<EntityWithSummary>) -> Self {
        Self(inner)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let inner = Vec::with_capacity(capacity);
        Self(inner)
    }
}

impl From<EntityCollector> for Vec<EntityWithSummary> {
    fn from(from: EntityCollector) -> Self {
        let EntityCollector(inner) = from;
        inner
    }
}

impl RecordCollector for EntityCollector {
    type Header = RecordHeader;
    type Record = (_core::Entity, Option<_core::Summary>);

    fn collect(
        &mut self,
        _header: RecordHeader,
        (entity, summary): (_core::Entity, Option<_core::Summary>),
    ) {
        let Self(inner) = self;
        inner.push(merge_entity_with_summary(
            entity.into(),
            summary.map(Into::into),
        ));
    }
}

impl ReservableRecordCollector for EntityCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}

fn merge_entity_with_summary(entity: Entity, summary: Option<Summary>) -> EntityWithSummary {
    let GenericEntity(hdr, body) = entity;
    let body = CollectionWithSummary {
        collection: body,
        summary,
    };
    GenericEntity(hdr, body)
}
