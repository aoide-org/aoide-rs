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

pub mod json;

mod _serde {
    pub use aoide_core_serde::playlist::Playlist;
}

use aoide_core_serde::playlist::PlaylistEntry;

use aoide_core::{
    entity::{EntityHeader, EntityRevision, EntityRevisionUpdateResult, EntityUid},
    playlist::{Entity, Playlist, PlaylistBrief},
};

use aoide_repo::{
    entity::{EntityBodyData, EntityData, EntityDataFormat, EntityDataVersion},
    playlist::Repo as _,
    Pagination, RepoResult,
};

use aoide_repo_sqlite::Connection as DbConnection;

use serde::Deserialize;

///////////////////////////////////////////////////////////////////////

const ENTITY_DATA_FORMAT: EntityDataFormat = EntityDataFormat::JSON;

const ENTITY_DATA_VERSION: EntityDataVersion = EntityDataVersion { major: 0, minor: 0 };

pub fn write_json_body_data(playlist: &_serde::Playlist) -> Fallible<EntityBodyData> {
    Ok((
        ENTITY_DATA_FORMAT,
        ENTITY_DATA_VERSION,
        serde_json::to_vec(playlist)?,
    ))
}

fn read_json_entity(entity_data: EntityData) -> Fallible<Entity> {
    let (hdr, json_data) = load_json_entity_data(entity_data)?;
    let playlist: _serde::Playlist = serde_json::from_slice(&json_data)?;
    Ok(Entity::new(hdr, Playlist::from(playlist)))
}

pub fn load_json_entity_data(entity_data: EntityData) -> Fallible<(EntityHeader, Vec<u8>)> {
    let (hdr, (data_fmt, data_ver, json_data)) = entity_data;
    if data_fmt != ENTITY_DATA_FORMAT {
        let e = anyhow!(
            "Unsupported data format when loading playlist {}: expected = {:?}, actual = {:?}",
            hdr.uid,
            ENTITY_DATA_FORMAT,
            data_fmt
        );
        return Err(e);
    }
    if data_ver < ENTITY_DATA_VERSION {
        // TODO: Data migration from an older version
        unimplemented!();
    }
    if data_ver == ENTITY_DATA_VERSION {
        return Ok((hdr, json_data));
    }
    let e = anyhow!(
        "Unsupported data version when loading playlist {}: expected = {:?}, actual = {:?}",
        hdr.uid,
        ENTITY_DATA_VERSION,
        data_ver
    );
    Err(e)
}

pub fn create_playlist(
    db: &SqlitePooledConnection,
    new_playlist: Playlist,
    body_data: EntityBodyData,
) -> RepoResult<Entity> {
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr, new_playlist);
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.insert_playlist(&entity, body_data).map(|()| entity)
    })
}

pub fn update_playlist(
    db: &SqlitePooledConnection,
    entity: &Entity,
    body_data: EntityBodyData,
) -> RepoResult<EntityRevisionUpdateResult> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.update_playlist(entity, body_data)
    })
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum PlaylistPatchOperation {
    AppendEntries {
        entries: Vec<PlaylistEntry>,
    },
    InsertEntries {
        before: usize,
        entries: Vec<PlaylistEntry>,
    },
    ReplaceEntries {
        start: Option<usize>,
        end: Option<usize>,
        entries: Vec<PlaylistEntry>,
    },
    RemoveEntries {
        start: Option<usize>,
        end: Option<usize>,
    },
    RemoveAllEntries,
    ReverseEntries,
    ShuffleEntries,
    SortEntriesChronologically,
}

pub fn patch_playlist(
    db: &SqlitePooledConnection,
    uid: &EntityUid,
    rev: Option<EntityRevision>,
    operations: impl IntoIterator<Item = PlaylistPatchOperation>,
) -> RepoResult<(EntityRevisionUpdateResult, Option<Playlist>)> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        let entity_data = repo.load_playlist(uid)?;
        if let Some(entity_data) = entity_data {
            let Entity {
                hdr,
                body: mut playlist,
                ..
            } = read_json_entity(entity_data)?;
            debug_assert_eq!(uid, &hdr.uid);
            if let Some(rev) = rev {
                if rev != hdr.rev {
                    // Conflicting revision
                    return Ok((EntityRevisionUpdateResult::Current(hdr.rev), Some(playlist)));
                }
            }
            let mut modified = false;
            for operation in operations.into_iter() {
                use PlaylistPatchOperation::*;
                match operation {
                    AppendEntries { entries } => {
                        if entries.is_empty() {
                            continue;
                        }
                        playlist.append_entries(entries.into_iter().map(Into::into));
                        modified = true;
                    }
                    InsertEntries { before, entries } => {
                        if entries.is_empty() {
                            continue;
                        }
                        let before = before.min(playlist.entries.len());
                        playlist.insert_entries(before, entries.into_iter().map(Into::into));
                        modified = true;
                    }
                    ReplaceEntries {
                        start,
                        end,
                        entries,
                    } => {
                        if playlist.entries.is_empty() {
                            continue;
                        }
                        let entries = entries.into_iter().map(Into::into);
                        match (start, end) {
                            (None, None) => {
                                playlist.replace_entries(.., entries);
                                modified = true;
                            }
                            (Some(start), None) => {
                                if start >= playlist.entries.len() {
                                    continue;
                                }
                                playlist.replace_entries(start.., entries);
                                modified = true;
                            }
                            (None, Some(end)) => {
                                let end = end.max(playlist.entries.len());
                                if end == 0 {
                                    continue;
                                }
                                playlist.replace_entries(..end, entries);
                                modified = true;
                            }
                            (Some(start), Some(end)) => {
                                let start = start.min(playlist.entries.len());
                                let end = end.max(start);
                                debug_assert!(start <= end);
                                if start == end {
                                    continue;
                                }
                                playlist.replace_entries(start..end, entries);
                                modified = true;
                            }
                        }
                    }
                    RemoveEntries { start, end } => {
                        if playlist.entries.is_empty() {
                            continue;
                        }
                        match (start, end) {
                            (None, None) => {
                                playlist.remove_all_entries();
                                modified = true;
                            }
                            (Some(start), None) => {
                                if start >= playlist.entries.len() {
                                    continue;
                                }
                                playlist.remove_entries(start..);
                                modified = true;
                            }
                            (None, Some(end)) => {
                                let end = end.max(playlist.entries.len());
                                if end == 0 {
                                    continue;
                                }
                                playlist.remove_entries(..end);
                                modified = true;
                            }
                            (Some(start), Some(end)) => {
                                let start = start.min(playlist.entries.len());
                                let end = end.max(start);
                                debug_assert!(start <= end);
                                if start == end {
                                    continue;
                                }
                                playlist.remove_entries(start..end);
                                modified = true;
                            }
                        }
                    }
                    RemoveAllEntries => {
                        if playlist.entries.is_empty() {
                            continue;
                        }
                        playlist.remove_all_entries();
                        modified = true;
                    }
                    ReverseEntries => {
                        if playlist.entries.is_empty() {
                            continue;
                        }
                        playlist.reverse_entries();
                        modified = true;
                    }
                    ShuffleEntries => {
                        if playlist.entries.is_empty() {
                            continue;
                        }
                        playlist.shuffle_entries();
                        modified = true;
                    }
                    SortEntriesChronologically => {
                        if playlist.entries.is_empty() {
                            continue;
                        }
                        playlist.sort_entries_chronologically();
                        modified = true;
                    }
                }
            }
            if !modified {
                return Ok((EntityRevisionUpdateResult::Current(hdr.rev), Some(playlist)));
            }
            let updated_body_data = json::serialize_entity_body_data(&playlist.clone().into())?;
            let updated_entity = Entity::new(hdr, playlist);
            repo.update_playlist(&updated_entity, updated_body_data)
                .map(|res| (res, Some(updated_entity.body)))
        } else {
            Ok((EntityRevisionUpdateResult::NotFound, None))
        }
    })
}

pub fn delete_playlist(db: &SqlitePooledConnection, uid: &EntityUid) -> RepoResult<Option<()>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.delete_playlist(uid)
    })
}

pub fn load_playlist(
    db: &SqlitePooledConnection,
    uid: &EntityUid,
) -> RepoResult<Option<EntityData>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.load_playlist(uid)
    })
}

pub fn list_playlists(
    db: &SqlitePooledConnection,
    r#type: Option<&str>,
    pagination: Pagination,
) -> RepoResult<Vec<EntityData>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.list_playlists(r#type, pagination)
    })
}

pub fn list_playlist_briefs(
    db: &SqlitePooledConnection,
    r#type: Option<&str>,
    pagination: Pagination,
) -> RepoResult<Vec<(EntityHeader, PlaylistBrief)>> {
    db.transaction::<_, Error, _>(|| {
        let repo = DbConnection::from_inner(&*db);
        repo.list_playlist_briefs(r#type, pagination)
    })
}
