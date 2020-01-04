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

use aoide_core::playlist::Entity as PlaylistEntity;

use aoide_repo::{entity::Repo as EntityRepo, RepoId, RepoResult};

use std::collections::{hash_map, HashMap};

///////////////////////////////////////////////////////////////////////
// RepositoryHelper
///////////////////////////////////////////////////////////////////////

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct RepositoryHelper<'a> {
    connection: &'a diesel::SqliteConnection,
}

impl<'a> RepositoryHelper<'a> {
    pub fn new(connection: &'a diesel::SqliteConnection) -> Self {
        Self { connection }
    }

    fn cleanup_brief(&self) -> RepoResult<()> {
        let query = diesel::delete(aux_playlist_brief::table.filter(
            aux_playlist_brief::playlist_id.ne_all(tbl_playlist::table.select(tbl_playlist::id)),
        ));
        query.execute(self.connection)?;
        Ok(())
    }

    fn delete_brief(&self, repo_id: RepoId) -> RepoResult<()> {
        let query = diesel::delete(
            aux_playlist_brief::table.filter(aux_playlist_brief::playlist_id.eq(repo_id)),
        );
        query.execute(self.connection)?;
        Ok(())
    }

    fn insert_brief(&self, repo_id: RepoId, playlist: &'a PlaylistBrief<'a>) -> RepoResult<()> {
        let insertable = playlist::InsertableBrief::bind(repo_id, playlist);
        let query = diesel::insert_into(aux_playlist_brief::table).values(&insertable);
        query.execute(self.connection)?;
        Ok(())
    }

    fn cleanup_tracks(&self) -> RepoResult<()> {
        // Orphaned tracks from entries with unknown playlist
        diesel::delete(aux_playlist_track::table.filter(
            aux_playlist_track::playlist_id.ne_all(tbl_playlist::table.select(tbl_playlist::id)),
        ))
        .execute(self.connection)?;
        // Orphaned tracks from entries with invalid reference count
        diesel::delete(aux_playlist_track::table.filter(aux_playlist_track::track_ref_count.le(0)))
            .execute(self.connection)?;
        Ok(())
    }

    fn delete_tracks(&self, repo_id: RepoId) -> RepoResult<()> {
        diesel::delete(
            aux_playlist_track::table.filter(aux_playlist_track::playlist_id.eq(repo_id)),
        )
        .execute(self.connection)?;
        Ok(())
    }

    fn insert_tracks(&self, repo_id: RepoId, playlist_entries: &[PlaylistEntry]) -> RepoResult<()> {
        let mut tracks: HashMap<EntityUid, usize> =
            HashMap::with_capacity(playlist_entries.len() + 100);
        for entry in playlist_entries {
            match tracks.entry(entry.track_uid.clone()) {
                hash_map::Entry::Occupied(mut entry) => {
                    debug_assert!(*entry.get() > 0);
                    *entry.get_mut() += 1;
                }
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(1);
                }
            }
        }
        debug_assert!(tracks.len() <= playlist_entries.len());
        for (track_uid, ref_count) in tracks {
            let insertable = playlist::InsertableTrack::bind(repo_id, &track_uid, ref_count);
            let query = diesel::insert_into(aux_playlist_track::table).values(&insertable);
            query.execute(self.connection)?;
        }
        Ok(())
    }

    pub fn cleanup(&self) -> RepoResult<()> {
        self.cleanup_tracks()?;
        self.cleanup_brief()?;
        Ok(())
    }

    fn on_insert(&self, repo_id: RepoId, playlist: &Playlist) -> RepoResult<()> {
        self.insert_brief(repo_id, &playlist.brief())?;
        self.insert_tracks(repo_id, &playlist.entries)?;
        Ok(())
    }

    fn on_delete(&self, repo_id: RepoId) -> RepoResult<()> {
        self.delete_tracks(repo_id)?;
        self.delete_brief(repo_id)?;
        Ok(())
    }

    fn on_refresh(&self, repo_id: RepoId, playlist: &Playlist) -> RepoResult<()> {
        self.on_delete(repo_id)?;
        self.on_insert(repo_id, playlist)?;
        Ok(())
    }

    pub fn refresh_entity(&self, entity: &PlaylistEntity) -> RepoResult<RepoId> {
        let uid = &entity.hdr.uid;
        match self.resolve_repo_id(uid)? {
            Some(repo_id) => {
                self.on_refresh(repo_id, &entity.body)?;
                Ok(repo_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn after_entity_inserted(&self, entity: &PlaylistEntity) -> RepoResult<RepoId> {
        let uid = &entity.hdr.uid;
        match self.resolve_repo_id(uid)? {
            Some(repo_id) => {
                self.on_insert(repo_id, &entity.body)?;
                Ok(repo_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn before_entity_updated_or_removed(&self, uid: &EntityUid) -> RepoResult<RepoId> {
        match self.resolve_repo_id(uid)? {
            Some(repo_id) => {
                self.on_delete(repo_id)?;
                Ok(repo_id)
            }
            None => Err(failure::format_err!("Entity not found: {}", uid)),
        }
    }

    pub fn after_entity_updated(&self, repo_id: RepoId, playlist: &Playlist) -> RepoResult<()> {
        self.on_insert(repo_id, playlist)?;
        Ok(())
    }
}

impl<'a> EntityRepo for RepositoryHelper<'a> {
    fn resolve_repo_id(&self, uid: &EntityUid) -> RepoResult<Option<RepoId>> {
        tbl_playlist::table
            .select(tbl_playlist::id)
            .filter(tbl_playlist::uid.eq(uid.as_ref()))
            .first::<RepoId>(self.connection)
            .optional()
            .map_err(Into::into)
    }
}
