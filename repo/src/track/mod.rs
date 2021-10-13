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

use aoide_core::{
    entity::{EntityHeader, EntityRevision, EntityUid},
    track::{Entity, Track},
    util::clock::DateTime,
};

use aoide_core_ext::track::search::*;

use crate::{
    collection::RecordId as CollectionId,
    media::source::{RecordId as MediaSourceId, Repo as MediaSourceRepo},
    prelude::*,
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

pub trait EntityRepo: MediaSourceRepo {
    fn resolve_track_id(&self, uid: &EntityUid) -> RepoResult<RecordId> {
        self.resolve_track_entity_revision(uid)
            .map(|(hdr, _rev)| hdr.id)
    }

    fn resolve_track_entity_revision(
        &self,
        uid: &EntityUid,
    ) -> RepoResult<(RecordHeader, EntityRevision)>;

    fn load_track_entity(&self, id: RecordId) -> RepoResult<(RecordHeader, Entity)>;

    fn load_track_entity_by_uid(&self, uid: &EntityUid) -> RepoResult<(RecordHeader, Entity)>;

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

    fn list_track_entities(
        &self,
        pagination: &Pagination,
    ) -> RepoResult<Vec<(RecordHeader, Entity)>>;

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

    fn delete_track_entity(&self, id: RecordId) -> RepoResult<()>;

    fn replace_collected_track_by_media_source_path(
        &self,
        collection_id: CollectionId,
        preserve_collected_at: bool,
        replace_mode: ReplaceMode,
        track: Track,
    ) -> RepoResult<ReplaceOutcome>;

    fn purge_tracks_by_media_source_media_source_path_predicate(
        &self,
        collection_id: CollectionId,
        media_source_path_predicate: StringPredicateBorrowed<'_>,
    ) -> RepoResult<usize>;

    fn purge_tracks_by_media_sources(
        &self,
        media_source_ids: &[MediaSourceId],
    ) -> RepoResult<usize>;

    fn search_collected_tracks(
        &self,
        collection_id: CollectionId,
        pagination: &Pagination,
        filter: Option<SearchFilter>,
        ordering: Vec<SortOrder>,
        collector: &mut dyn ReservableRecordCollector<Header = RecordHeader, Record = Entity>,
    ) -> RepoResult<usize>;

    fn count_collected_tracks(&self, collection_id: CollectionId) -> RepoResult<u64>;
}
