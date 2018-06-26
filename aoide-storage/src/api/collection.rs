// Aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom>
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

use api::Pagination;

use aoide_core::audio::DurationMs;
use aoide_core::domain::collection::*;
use aoide_core::domain::entity::*;

pub type CollectionsResult<T> = Result<T, Error>;

pub trait Collections {
    fn create_entity(&self, body: Collection) -> CollectionsResult<CollectionEntity>;

    fn insert_entity(&self, entity: &CollectionEntity) -> CollectionsResult<()>;

    fn update_entity(
        &self,
        entity: &CollectionEntity,
    ) -> CollectionsResult<(EntityRevision, Option<EntityRevision>)>;

    fn delete_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<()>>;

    fn load_entity(&self, uid: &EntityUid) -> CollectionsResult<Option<CollectionEntity>>;

    fn list_entities(&self, pagination: &Pagination) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_starting_with(
        &self,
        name: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_containing(
        &self,
        name: &str,
        pagination: &Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionTrackStats {
    pub total_count: usize,
    pub total_duration_ms: DurationMs,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionStats {
    pub tracks: Option<CollectionTrackStats>,
}

impl CollectionStats {
    pub fn is_empty(&self) -> bool {
        self.tracks.is_none()
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionEntityWithStats {
    #[serde(flatten)]
    pub entity: CollectionEntity,

    #[serde(skip_serializing_if = "CollectionStats::is_empty", default)]
    pub stats: CollectionStats,
}
