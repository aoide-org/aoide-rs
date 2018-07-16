// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use failure::Error;

use api::{
    collection::CollectionTrackStats, LocateTracksParams, Pagination, ReplaceTracksParams,
    ReplacedTracks, ScoredTagCount, SearchTracksParams, StringField, StringFieldCounts,
    TagFacetCount,
};

use super::serde::{SerializationFormat, SerializedEntity};

use aoide_core::domain::entity::*;
use aoide_core::domain::track::*;

pub type TracksResult<T> = Result<T, Error>;

pub trait Tracks {
    fn create_entity(&self, body: Track, format: SerializationFormat) -> TracksResult<TrackEntity>;

    fn insert_entity(&self, entity: &TrackEntity, format: SerializationFormat) -> TracksResult<()>;

    fn update_entity(
        &self,
        entity: TrackEntity,
        format: SerializationFormat,
    ) -> TracksResult<(EntityRevision, Option<EntityRevision>)>;

    fn replace_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        replace_params: ReplaceTracksParams,
        format: SerializationFormat,
    ) -> TracksResult<ReplacedTracks>;

    fn delete_entity(&self, uid: &EntityUid) -> TracksResult<Option<()>>;

    fn load_entity(&self, uid: &EntityUid) -> TracksResult<Option<SerializedEntity>>;

    fn locate_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        locate_params: LocateTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>>;

    fn search_entities(
        &self,
        collection_uid: Option<&EntityUid>,
        pagination: Pagination,
        search_params: SearchTracksParams,
    ) -> TracksResult<Vec<SerializedEntity>>;

    fn list_fields(
        &self,
        collection_uid: Option<&EntityUid>,
        field: StringField,
        pagination: Pagination,
    ) -> TracksResult<StringFieldCounts>;

    fn collection_stats(&self, collection_uid: &EntityUid) -> TracksResult<CollectionTrackStats>;
}

pub type TrackTagsResult<T> = Result<T, Error>;

pub trait TrackTags {
    fn list_tag_facets(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<TagFacetCount>>;

    fn list_tags(
        &self,
        collection_uid: Option<&EntityUid>,
        facets: Option<&Vec<&str>>,
        pagination: Pagination,
    ) -> TrackTagsResult<Vec<ScoredTagCount>>;
}
