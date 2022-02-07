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
    entity::{EntityHeader, EntityRevision, EntityUid},
    media::SourcePath,
    track::{Entity, Track},
    util::clock::DateTime,
};

use aoide_core_api::track::search::*;

use crate::{
    collection::RecordId as CollectionId, media::source::RecordId as MediaSourceId, prelude::*,
};

record_id_newtype!(RecordId);

pub type RecordHeader = crate::RecordHeader<RecordId>;

#[derive(Clone, Debug, Eq, PartialEq)]
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
    pub media_source_path: SourcePath,
    pub media_source_synchronized_at: Option<DateTime>,
    pub media_source_synchronized_rev: Option<EntityRevision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplaceParams {
    pub mode: ReplaceMode,
    pub preserve_collected_at: bool,
    pub update_media_source_synchronized_rev: bool,
}

pub trait EntityRepo {
    fn resolve_track_id(&self, uid: &EntityUid) -> RepoResult<RecordId>;

    fn load_track_entity(&self, id: RecordId) -> RepoResult<(RecordHeader, Entity)>;

    fn load_track_entity_by_uid(&self, uid: &EntityUid) -> RepoResult<(RecordHeader, Entity)>;

    fn insert_track_entity(
        &self,
        created_at: DateTime,
        media_source_id: MediaSourceId,
        created_entity: &Entity,
    ) -> RepoResult<RecordId>;

    fn update_track_entity(
        &self,
        id: RecordId,
        updated_at: DateTime,
        media_source_id: MediaSourceId,
        updated_entity: &Entity,
    ) -> RepoResult<()>;

    fn purge_track_entity(&self, id: RecordId) -> RepoResult<()>;
}

pub trait CollectionRepo {
    fn load_track_entity_by_media_source_path(
        &self,
        collection_id: CollectionId,
        media_source_path: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, Entity)>;

    fn resolve_track_entity_header_by_media_source_path(
        &self,
        collection_id: CollectionId,
        media_source_path: &str,
    ) -> RepoResult<(MediaSourceId, RecordHeader, EntityHeader)>;

    fn replace_track_by_media_source_path(
        &self,
        collection_id: CollectionId,
        params: ReplaceParams,
        track: Track,
    ) -> RepoResult<ReplaceOutcome>;

    fn search_tracks(
        &self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<SearchFilter>,
        ordering: Vec<SortOrder>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
    ) -> RepoResult<usize>;

    fn count_tracks(&self, collection_id: CollectionId) -> RepoResult<u64>;

    fn purge_tracks_by_media_source_path_predicate(
        &self,
        collection_id: CollectionId,
        media_source_path_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize>;

    fn find_unsynchronized_tracks(
        &self,
        collection_id: CollectionId,
        pagination: &Pagination,
        media_source_path_predicate: Option<StringPredicateBorrowed<'_>>,
    ) -> RepoResult<Vec<(EntityHeader, RecordHeader, RecordTrail)>>;
}
