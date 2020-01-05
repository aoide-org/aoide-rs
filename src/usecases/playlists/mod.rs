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

mod _core {
    pub use aoide_core::playlist::Playlist;
}

use aoide_core_serde::playlist::Playlist;

use aoide_core::{
    entity::{EntityHeader, EntityRevision, EntityRevisionUpdateResult, EntityUid},
    playlist::Entity,
};

use aoide_repo::{
    entity::{EntityBodyData, EntityData, EntityDataFormat, EntityDataVersion},
    playlist::Repo as _,
    Pagination, RepoResult,
};

use aoide_repo_sqlite::playlist::Repository;

use futures::future::{self, Future};
use serde::Deserialize;

///////////////////////////////////////////////////////////////////////

const ENTITY_DATA_FORMAT: EntityDataFormat = EntityDataFormat::JSON;

const ENTITY_DATA_VERSION: EntityDataVersion = EntityDataVersion { major: 0, minor: 0 };

pub fn write_json_body_data(playlist: &Playlist) -> Fallible<EntityBodyData> {
    Ok((
        ENTITY_DATA_FORMAT,
        ENTITY_DATA_VERSION,
        serde_json::to_vec(playlist)?,
    ))
}

fn read_json_entity(entity_data: EntityData) -> Fallible<Entity> {
    let (hdr, json_data) = load_json_entity_data(entity_data)?;
    let playlist: Playlist = serde_json::from_slice(&json_data)?;
    Ok(Entity::new(hdr, _core::Playlist::from(playlist)))
}

pub fn load_json_entity_data(entity_data: EntityData) -> Fallible<(EntityHeader, Vec<u8>)> {
    let (hdr, (data_fmt, data_ver, json_data)) = entity_data;
    if data_fmt != ENTITY_DATA_FORMAT {
        let e = failure::format_err!(
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
    let e = failure::format_err!(
        "Unsupported data version when loading playlist {}: expected = {:?}, actual = {:?}",
        hdr.uid,
        ENTITY_DATA_VERSION,
        data_ver
    );
    Err(e)
}

pub fn create_playlist(
    conn: &SqlitePooledConnection,
    new_playlist: _core::Playlist,
    body_data: EntityBodyData,
) -> RepoResult<EntityHeader> {
    let repo = Repository::new(&*conn);
    let hdr = EntityHeader::initial_random();
    let entity = Entity::new(hdr.clone(), new_playlist);
    conn.transaction::<_, Error, _>(|| repo.insert_playlist(entity, body_data).map(|()| hdr))
}

pub fn update_playlist(
    conn: &SqlitePooledConnection,
    entity: Entity,
    body_data: EntityBodyData,
) -> RepoResult<EntityRevisionUpdateResult> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.update_playlist(entity, body_data))
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum PlaylistPatchAction {
    SortEntriesChronologically,
}

pub fn patch_playlist(
    conn: &SqlitePooledConnection,
    uid: &EntityUid,
    rev: Option<EntityRevision>,
    action: PlaylistPatchAction,
) -> RepoResult<EntityRevisionUpdateResult> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| {
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
                    return Ok(EntityRevisionUpdateResult::CurrentIsNewer(hdr.rev));
                }
            }
            use PlaylistPatchAction::*;
            match action {
                SortEntriesChronologically => playlist.sort_entries_chronologically(),
            }
            let updated_body_data = json::serialize_entity_body_data(&playlist.clone().into())?;
            let updated_entity = Entity::new(hdr, playlist);
            repo.update_playlist(updated_entity, updated_body_data)
        } else {
            Ok(EntityRevisionUpdateResult::NotFound)
        }
    })
}

pub fn delete_playlist(conn: &SqlitePooledConnection, uid: &EntityUid) -> RepoResult<Option<()>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.delete_playlist(uid))
}

pub fn load_playlist(
    conn: &SqlitePooledConnection,
    uid: &EntityUid,
) -> RepoResult<Option<EntityData>> {
    let repo = Repository::new(&*conn);
    conn.transaction::<_, Error, _>(|| repo.load_playlist(uid))
}

pub fn list_playlists(
    conn: &SqlitePooledConnection,
    r#type: Option<&str>,
    pagination: Pagination,
) -> impl Future<Item = Vec<EntityData>, Error = Error> {
    let repo = Repository::new(&*conn);
    future::result(conn.transaction::<_, Error, _>(|| repo.list_playlists(r#type, pagination)))
}
