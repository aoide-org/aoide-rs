// aoide.org - Copyright (C) 2018-2020 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core::{
    collection::*,
    entity::{EntityRevision, EntityUid},
};

pub trait Repo {
    fn resolve_collection_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>>;

    fn insert_collection(&self, entity: &Entity) -> RepoResult<()>;

    fn update_collection(
        &self,
        entity: &Entity,
    ) -> RepoResult<(EntityRevision, Option<EntityRevision>)>;

    fn delete_collection(&self, uid: &EntityUid) -> RepoResult<Option<()>>;

    fn load_collection(&self, uid: &EntityUid) -> RepoResult<Option<Entity>>;

    fn list_collections(&self, pagination: Pagination) -> RepoResult<Vec<Entity>>;

    fn find_collections_by_name(&self, name: &str) -> RepoResult<Vec<Entity>>;

    fn find_collections_by_name_starting_with(
        &self,
        name: &str,
        pagination: Pagination,
    ) -> RepoResult<Vec<Entity>>;

    fn find_collections_by_name_containing(
        &self,
        name: &str,
        pagination: Pagination,
    ) -> RepoResult<Vec<Entity>>;
}

pub trait TrackEntryRepo {
    /// Insert or update the given entry
    fn replace_track_entry(
        &self,
        collection_uid: &EntityUid,
        track_uid: &EntityUid,
        entry: SingleTrackEntry,
    ) -> RepoResult<()>;

    /// Remove the given entry
    fn remove_track_entry(
        &self,
        collection_uid: &EntityUid,
        track_uid: &EntityUid,
    ) -> RepoResult<bool>;

    /// Remove the given entry
    fn remove_all_track_entries(&self, collection_uid: &EntityUid) -> RepoResult<usize>;

    /// Try to load the given entry if it exists
    fn load_track_entry(
        &self,
        collection_uid: &EntityUid,
        track_uid: &EntityUid,
    ) -> RepoResult<Option<SingleTrackEntry>>;
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrackStats {
    pub total_count: usize,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct Stats {
    pub tracks: Option<TrackStats>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EntityWithStats {
    pub entity: Entity,
    pub stats: Stats,
}
