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

use aoide_core::{
    collection::{Entity, EntityHeader, EntityUid},
    util::{clock::DateTime, url::BaseUrl},
};

use aoide_core_api::collection::{EntityWithSummary, Summary};

use crate::prelude::*;

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

#[derive(Debug, Clone)]
pub enum MediaSourceRootUrlFilter {
    Equal(Option<BaseUrl>),
    Prefix(BaseUrl),
    PrefixOf(BaseUrl),
}

pub trait EntityRepo {
    entity_repo_trait_common_functions!(RecordId, Entity, EntityUid, EntityHeader, Collection);

    fn insert_collection_entity(
        &self,
        created_at: DateTime,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;

    fn load_collection_entities(
        &self,
        kind: Option<&str>,
        media_source_root_url: Option<&MediaSourceRootUrlFilter>,
        with_summary: bool,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<
            Header = RecordHeader,
            Record = EntityWithSummary,
        >,
    ) -> RepoResult<()>;

    fn load_collection_summary(&self, id: RecordId) -> RepoResult<Summary>;

    fn load_all_kinds(&self) -> RepoResult<Vec<String>>;
}

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
    type Record = EntityWithSummary;

    fn collect(&mut self, _header: RecordHeader, record: EntityWithSummary) {
        let Self(inner) = self;
        inner.push(record);
    }
}

impl ReservableRecordCollector for EntityWithSummaryCollector {
    fn reserve(&mut self, additional: usize) {
        let Self(inner) = self;
        inner.reserve(additional);
    }
}
