// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::borrow::Cow;

use aoide_core::{
    CollectionEntity, CollectionHeader, CollectionUid,
    util::{clock::UtcDateTimeMs, url::BaseUrl},
};
use aoide_core_api::{
    Pagination,
    collection::{EntityWithSummary, LoadScope, Summary},
};

use crate::{RecordCollector, RepoResult, ReservableRecordCollector};

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

#[derive(Debug, Clone)]
pub enum MediaSourceRootUrlFilter {
    IsNone,
    Equal(BaseUrl),
    NotEqual(BaseUrl),
    Prefix(BaseUrl),
    PrefixOf(BaseUrl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KindFilter<'a> {
    IsNone,
    Equal(Cow<'a, str>),
    NotEqual(Cow<'a, str>),
}

pub trait EntityRepo {
    entity_repo_trait_common_functions!(
        RecordId,
        CollectionEntity,
        CollectionUid,
        CollectionHeader,
        Collection
    );

    fn insert_collection_entity(
        &mut self,
        created_at: UtcDateTimeMs,
        created_entity: &CollectionEntity,
    ) -> RepoResult<RecordId>;

    fn load_collection_entities(
        &mut self,
        kind_filter: Option<KindFilter<'_>>,
        media_source_root_url: Option<&MediaSourceRootUrlFilter>,
        load_scope: LoadScope,
        pagination: Option<&Pagination>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = EntityWithSummary>,
    ) -> RepoResult<()>;

    fn load_collection_summary(&mut self, id: RecordId) -> RepoResult<Summary>;

    fn load_all_kinds(&mut self) -> RepoResult<Vec<String>>;
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
