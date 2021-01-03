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

use crate::track::schema::*;

use aoide_core::playlist::Entity as PlaylistEntity;

use aoide_repo::{track::EntityRepo as _, RecordId, RepoResult};

use std::collections::{hash_map, HashMap};

///////////////////////////////////////////////////////////////////////
// Utility functions
///////////////////////////////////////////////////////////////////////

fn cleanup_entries_brief<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    let query = diesel::delete(aux_playlist_entries::table.filter(
        aux_playlist_entries::playlist_id.ne_all(collection_playlist::table.select(collection_playlist::id)),
    ));
    query.execute(connection.as_ref())?;
    Ok(())
}

fn delete_entries_brief<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
) -> RepoResult<()> {
    let query = diesel::delete(
        aux_playlist_entries::table.filter(aux_playlist_entries::playlist_id.eq(record_id)),
    );
    query.execute(connection.as_ref())?;
    Ok(())
}

fn insert_entries_brief<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    playlist: &PlaylistEntriesSummary,
) -> RepoResult<()> {
    let insertable = playlist::InsertableEntriesSummary::bind(record_id, playlist);
    let query = diesel::insert_into(aux_playlist_entries::table).values(&insertable);
    query.execute(connection.as_ref())?;
    Ok(())
}

fn cleanup_tracks<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    // Orphaned tracks from entries with unknown playlist
    diesel::delete(
        aux_playlist_track_entries::table.filter(
            aux_playlist_track_entries::playlist_id
                .ne_all(collection_playlist::table.select(collection_playlist::id)),
        ),
    )
    .execute(connection.as_ref())?;
    // Orphaned tracks from entries with unknown playlist
    diesel::delete(aux_playlist_track_entries::table.filter(
        aux_playlist_track_entries::track_id.ne_all(track::table.select(track::row_id)),
    ))
    .execute(connection.as_ref())?;
    // Orphaned tracks from entries with invalid reference count
    diesel::delete(
        aux_playlist_track_entries::table.filter(aux_playlist_track_entries::track_ref_count.le(0)),
    )
    .execute(connection.as_ref())?;
    Ok(())
}

fn delete_tracks<'db>(connection: &crate::Connection<'db>, record_id: RecordId) -> RepoResult<()> {
    diesel::delete(
        aux_playlist_track_entries::table
            .filter(aux_playlist_track_entries::playlist_id.eq(record_id)),
    )
    .execute(connection.as_ref())?;
    Ok(())
}

fn insert_tracks<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    playlist_entries: &[PlaylistEntry],
) -> RepoResult<()> {
    let mut tracks: HashMap<RecordId, usize> = HashMap::with_capacity(playlist_entries.len() + 100);
    for playlist_entry in playlist_entries {
        use PlaylistItem::*;
        match &playlist_entry.item {
            Separator => {}
            Track(track) => {
                let track_id = connection.resolve_track_id(&track.uid)?;
                if let Some(track_id) = track_id {
                    match tracks.entry(track_id) {
                        hash_map::Entry::Occupied(mut entry) => {
                            debug_assert!(*entry.get() > 0);
                            *entry.get_mut() += 1;
                        }
                        hash_map::Entry::Vacant(entry) => {
                            entry.insert(1);
                        }
                    }
                } else {
                    log::info!("Skipping playlist entry with unknown track {}", track.uid);
                }
            }
        }
    }
    debug_assert!(tracks.len() <= playlist_entries.len());
    for (track_id, ref_count) in tracks {
        let insertable = playlist::InsertableTrackEntries::bind(record_id, track_id, ref_count);
        let query = diesel::insert_into(aux_playlist_track_entries::table).values(&insertable);
        query.execute(connection.as_ref())?;
    }
    Ok(())
}

pub fn cleanup<'db>(connection: &crate::Connection<'db>) -> RepoResult<()> {
    cleanup_tracks(connection)?;
    cleanup_entries_brief(connection)?;
    Ok(())
}

fn on_insert<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    playlist: &Playlist,
) -> RepoResult<()> {
    insert_entries_brief(connection, record_id, &playlist.entries_brief())?;
    insert_tracks(connection, record_id, &playlist.entries)?;
    Ok(())
}

fn on_delete<'db>(connection: &crate::Connection<'db>, record_id: RecordId) -> RepoResult<()> {
    delete_tracks(connection, record_id)?;
    delete_entries_brief(connection, record_id)?;
    Ok(())
}

fn on_refresh<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    playlist: &Playlist,
) -> RepoResult<()> {
    on_delete(connection, record_id)?;
    on_insert(connection, record_id, playlist)?;
    Ok(())
}

pub fn refresh_entity<'db>(
    connection: &crate::Connection<'db>,
    entity: &PlaylistEntity,
) -> RepoResult<RecordId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_playlist_id(uid)? {
        Some(record_id) => {
            on_refresh(connection, record_id, &entity.body)?;
            Ok(record_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_inserted<'db>(
    connection: &crate::Connection<'db>,
    entity: &PlaylistEntity,
) -> RepoResult<RecordId> {
    let uid = &entity.hdr.uid;
    match connection.resolve_playlist_id(uid)? {
        Some(record_id) => {
            on_insert(connection, record_id, &entity.body)?;
            Ok(record_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn before_entity_updated_or_removed<'db>(
    connection: &crate::Connection<'db>,
    uid: &EntityUid,
) -> RepoResult<RecordId> {
    match connection.resolve_playlist_id(uid)? {
        Some(record_id) => {
            on_delete(connection, record_id)?;
            Ok(record_id)
        }
        None => Err(anyhow!("Entity not found: {}", uid)),
    }
}

pub fn after_entity_updated<'db>(
    connection: &crate::Connection<'db>,
    record_id: RecordId,
    playlist: &Playlist,
) -> RepoResult<()> {
    on_insert(connection, record_id, playlist)?;
    Ok(())
}
