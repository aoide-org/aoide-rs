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

use aoide_core::collection::EntityUid;

use aoide_repo::{
    collection::RecordHeader,
    prelude::{RecordCollector, ReservableRecordCollector},
};

use aoide_core_json::collection::{Collection, Entity};

use aoide_core_api_json::collection::{export_entity_with_summary, EntityWithSummary};

use super::*;

mod _inner {
    pub(super) use aoide_core::collection::EntityHeader;
    pub(super) use aoide_core_api::collection::EntityWithSummary;
}

pub mod create;
pub mod load_all;
pub mod load_all_kinds;
pub mod load_one;
pub mod purge;
pub mod update;

#[derive(Debug, Default)]
pub struct EntityWithSummaryCollector(Vec<EntityWithSummary>);

impl EntityWithSummaryCollector {
    #[must_use]
    pub const fn new(inner: Vec<EntityWithSummary>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn finish(self) -> Vec<EntityWithSummary> {
        let Self(inner) = self;
        inner
    }
}

impl RecordCollector for EntityWithSummaryCollector {
    type Header = RecordHeader;
    type Record = _inner::EntityWithSummary;

    fn collect(&mut self, _header: RecordHeader, record: _inner::EntityWithSummary) {
        let Self(inner) = self;
        inner.push(export_entity_with_summary(record));
    }
}

impl ReservableRecordCollector for EntityWithSummaryCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
