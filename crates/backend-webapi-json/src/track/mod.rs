// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::EntityUid;

use aoide_repo::{
    prelude::{RecordCollector, ReservableRecordCollector},
    track::RecordHeader,
};

use aoide_core_json::track::Entity;

use super::*;

mod _core {
    pub(super) use aoide_core::track::Entity;
}

pub mod export_metadata;
pub mod find_unsynchronized;
pub mod import_and_replace;
pub mod load_many;
pub mod load_one;
pub mod replace;
pub mod resolve;
pub mod search;

const DEFAULT_PAGINATION: Pagination = Pagination {
    limit: Some(100),
    offset: None,
};

#[derive(Debug, Default)]
pub struct EntityCollector(Vec<Entity>);

impl EntityCollector {
    #[must_use]
    pub const fn new(inner: Vec<Entity>) -> Self {
        Self(inner)
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let inner = Vec::with_capacity(capacity);
        Self(inner)
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
    type Record = _core::Entity;

    fn collect(&mut self, _record_header: RecordHeader, entity: _core::Entity) {
        let Self(inner) = self;
        inner.push(entity.into());
    }
}

impl ReservableRecordCollector for EntityCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
