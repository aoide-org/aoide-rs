// aoide.org - Copyright (C) 2018-2019 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::*;

use crate::api::Pagination;

use failure::Error;

///////////////////////////////////////////////////////////////////////

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

    fn list_entities(&self, pagination: Pagination) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name(&self, name: &str) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_starting_with(
        &self,
        name: &str,
        pagination: Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;

    fn find_entities_by_name_containing(
        &self,
        name: &str,
        pagination: Pagination,
    ) -> CollectionsResult<Vec<CollectionEntity>>;
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionTrackStats {
    pub total_count: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionStats {
    pub tracks: Option<CollectionTrackStats>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CollectionEntityWithStats {
    pub entity: CollectionEntity,

    #[serde(skip_serializing_if = "IsDefault::is_default", default)]
    pub stats: CollectionStats,
}
