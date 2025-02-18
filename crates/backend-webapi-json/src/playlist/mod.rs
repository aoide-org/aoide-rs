// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::playlist::EntityUid;
use aoide_core_api_json::playlist::{EntityWithEntriesSummary, export_entity_with_entries_summary};
use aoide_core_json::playlist::{Entity, Entry, Playlist};
use aoide_repo::{RecordCollector, ReservableRecordCollector, playlist::RecordHeader};

use super::*;

mod _core {
    pub(super) use aoide_core::playlist::EntityHeader;
    pub(super) use aoide_core_api::playlist::EntityWithEntriesSummary;
}

pub mod create;
pub mod entries;
pub mod load_all;
pub mod load_one;
pub mod purge;
pub mod update;

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
    type Record = _core::EntityWithEntriesSummary;

    fn collect(&mut self, _header: RecordHeader, record: _core::EntityWithEntriesSummary) {
        let Self(inner) = self;
        inner.push(export_entity_with_entries_summary(record));
    }
}

impl ReservableRecordCollector for EntityWithEntriesSummaryCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
