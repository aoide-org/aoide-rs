// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{
    entity::EntityRevision,
    media::content::ContentLink,
    track::{Entity, EntityHeader, EntityUid, Track},
};
use aoide_core_api::track::search::{Filter, SortOrder, StringField};

use crate::{
    collection::RecordId as CollectionId, media::source::RecordId as MediaSourceId, prelude::*,
};

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringFieldCounts {
    pub field: StringField,
    pub counts: Vec<StringCount>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplaceMode {
    CreateOnly,
    UpdateOnly,
    UpdateOrCreate,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ReplaceOutcome {
    Created(MediaSourceId, RecordId, Entity),
    Updated(MediaSourceId, RecordId, Entity),
    Unchanged(MediaSourceId, RecordId, Entity),
    NotCreated(Track),
    NotUpdated(MediaSourceId, RecordId, Track),
}

/// Essential properties that allow to trace down a track from all
/// directions, i.e. database relations, source path, and source
/// synchronization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecordTrail {
    pub collection_id: CollectionId,
    pub media_source_id: MediaSourceId,
    pub content_link: ContentLink,
    pub last_synchronized_rev: Option<EntityRevision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaceParams {
    pub mode: ReplaceMode,
    pub preserve_collected_at: bool,
    pub update_last_synchronized_rev: bool,
}

pub trait EntityRepo {
    fn resolve_track_id(&mut self, uid: &EntityUid) -> RepoResult<RecordId>;

    fn load_track_entity(&mut self, id: RecordId) -> RepoResult<(RecordHeader, Entity)>;

    fn load_track_entity_by_uid(&mut self, uid: &EntityUid) -> RepoResult<(RecordHeader, Entity)>;

    fn insert_track_entity(
        &mut self,
        media_source_id: MediaSourceId,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;

    fn update_track_entity(
        &mut self,
        id: RecordId,
        media_source_id: MediaSourceId,
        updated_entity: &Entity,
    ) -> RepoResult<()>;

    fn purge_track_entity(&mut self, id: RecordId) -> RepoResult<()>;
}

pub trait CollectionRepo {
    fn load_track_entity_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, Entity)>;

    fn resolve_track_entity_header_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        content_path: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, EntityHeader)>;

    fn replace_track_by_media_source_content_path(
        &mut self,
        collection_id: CollectionId,
        params: ReplaceParams,
        track: Track,
    ) -> RepoResult<ReplaceOutcome>;

    fn search_tracks(
        &mut self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<Filter>,
        ordering: Vec<SortOrder>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
    ) -> RepoResult<usize>;

    fn count_tracks(&mut self, collection_id: CollectionId) -> RepoResult<u64>;

    fn purge_tracks_by_media_source_content_path_predicate(
        &mut self,
        collection_id: CollectionId,
        content_path_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize>;

    fn find_unsynchronized_tracks(
        &mut self,
        collection_id: CollectionId,
        pagination: &Pagination,
        content_path_predicate: Option<StringPredicateBorrowed<'_>>,
    ) -> RepoResult<Vec<(EntityHeader, RecordHeader, RecordTrail)>>;
}
