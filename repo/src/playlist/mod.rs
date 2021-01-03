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

use super::*;

use crate::entity::*;

use aoide_core::{
    entity::{EntityRevisionUpdateResult, EntityUid},
    playlist::*,
};

pub trait Repo {
    fn resolve_playlist_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>>;

    fn insert_playlist(&self, entity: &Entity, body_data: EntityBodyData) -> RepoResult<()>;

    fn update_playlist(
        &self,
        entity: &Entity,
        body_data: EntityBodyData,
    ) -> RepoResult<EntityRevisionUpdateResult>;

    fn delete_playlist(&self, uid: &EntityUid) -> RepoResult<Option<()>>;

    fn load_playlist(&self, uid: &EntityUid) -> RepoResult<Option<EntityData>>;

    fn load_playlist_rev(&self, hdr: &EntityHeader) -> RepoResult<Option<EntityData>>;

    fn list_playlists(
        &self,
        r#type: Option<&str>,
        pagination: Pagination,
    ) -> RepoResult<Vec<EntityData>>;

    fn list_playlist_briefs(
        &self,
        r#type: Option<&str>,
        pagination: Pagination,
    ) -> RepoResult<Vec<(EntityHeader, PlaylistBrief)>>;

    fn count_playlist_entries(&self, uid: &EntityUid) -> RepoResult<Option<usize>>;
}
