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

use aoide_repo::{RepoId, RepoResult};

use std::collections::{hash_map, HashMap};

///////////////////////////////////////////////////////////////////////
// Utility functions
///////////////////////////////////////////////////////////////////////

fn cleanup_brief<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    let query = diesel::delete(aux_playlist_brief::table.filter(
        aux_playlist_brief::playlist_id.ne_all(tbl_playlist::table.select(tbl_playlist::id)),
    ));
    query.execute(connection.as_ref())?;
    Ok(())
}

fn delete_brief<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    let query = diesel::delete(
        aux_playlist_brief::table.filter(aux_playlist_brief::playlist_id.eq(repo_id)),
    );
    query.execute(connection.as_ref())?;
    Ok(())
}

fn insert_brief<'db, 'a>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    playlist: &'a PlaylistBriefRef<'a>,
) -> RepoResult<()> {
    let insertable = playlist::InsertableBrief::bind(repo_id, playlist);
    let query = diesel::insert_into(aux_playlist_brief::table).values(&insertable);
    query.execute(connection.as_ref())?;
    Ok(())
}

fn cleanup_tracks<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    // Orphaned tracks from entries with unknown playlist
    diesel::delete(aux_playlist_track::table.filter(
        aux_playlist_track::playlist_id.ne_all(tbl_playlist::table.select(tbl_playlist::id)),
    ))
    .execute(connection.as_ref())?;
    // Orphaned tracks from entries with invalid reference count
    diesel::delete(aux_playlist_track::table.filter(aux_playlist_track::track_ref_count.le(0)))
        .execute(connection.as_ref())?;
    Ok(())
}

fn delete_tracks<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    diesel::delete(aux_playlist_track::table.filter(aux_playlist_track::playlist_id.eq(repo_id)))
        .execute(connection.as_ref())?;
    Ok(())
}

fn insert_tracks<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    playlist_entries: &[PlaylistEntry],
) -> RepoResult<()> {
    let mut tracks: HashMap<EntityUid, usize> =
        HashMap::with_capacity(playlist_entries.len() + 100);
    for playlist_entry in playlist_entries {
        use PlaylistItem::*;
        match &playlist_entry.item {
            Separator => {}
            Track(track) => match tracks.entry(track.uid.clone()) {
                hash_map::Entry::Occupied(mut entry) => {
                    debug_assert!(*entry.get() > 0);
                    *entry.get_mut() += 1;
                }
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(1);
                }
            },
        }
    }
    debug_assert!(tracks.len() <= playlist_entries.len());
    for (track_uid, ref_count) in tracks {
        let insertable = playlist::InsertableTrack::bind(repo_id, &track_uid, ref_count);
        let query = diesel::insert_into(aux_playlist_track::table).values(&insertable);
        query.execute(connection.as_ref())?;
    }
    Ok(())
}

pub fn cleanup<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    cleanup_tracks(connection)?;
    cleanup_brief(connection)?;
    Ok(())
}

fn on_insert<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    playlist: &Playlist,
) -> RepoResult<()> {
    insert_brief(connection, repo_id, &playlist.brief_ref())?;
    insert_tracks(connection, repo_id, &playlist.entries)?;
    Ok(())
}

fn on_delete<'db>(connection: &crate::Connection<'db>, repo_id: RepoId) -> RepoResult<()> {
    delete_tracks(connection, repo_id)?;
    delete_brief(connection, repo_id)?;
    Ok(())
}

fn on_refresh<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    playlist: &Playlist,
) -> RepoResult<()> {
    on_delete(connection, repo_id)?;
    on_insert(connection, repo_id, playlist)?;
    Ok(())
}

pub fn refresh_entity<'db>(
    connection: &crate::Connection<'db>,
    entity: &PlaylistEntity,
) -> RepoResult<RepoId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_playlist_id(uid)? {
        Some(repo_id) => {
            on_refresh(connection, repo_id, &entity.body)?;
            Ok(repo_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_inserted<'db>(
    connection: &crate::Connection<'db>,
    entity: &PlaylistEntity,
) -> RepoResult<RepoId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_playlist_id(uid)? {
        Some(repo_id) => {
            on_insert(connection, repo_id, &entity.body)?;
            Ok(repo_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn before_entity_updated_or_removed<'db>(
    connection: &crate::Connection<'db>,
    uid: &EntityUid,
) -> RepoResult<RepoId> {
    match connection.resolve_playlist_id(uid)? {
        Some(repo_id) => {
            on_delete(connection, repo_id)?;
            Ok(repo_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_updated<'db>(
    connection: &crate::Connection<'db>,
    repo_id: RepoId,
    playlist: &Playlist,
) -> RepoResult<()> {
    on_insert(connection, repo_id, playlist)?;
    Ok(())
}
