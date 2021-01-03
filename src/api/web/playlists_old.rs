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

mod json {
    pub use super::super::json::*;
    pub use crate::usecases::playlists::json::*;
}

use crate::usecases::playlists::*;

mod _serde {
    pub use aoide_core_serde::entity::{Entity, EntityHeader};
}

use aoide_core::{
    entity::{EntityHeader, EntityRevisionUpdateOutcome, EntityUid},
    playlist::Entity,
};

use aoide_core_serde::{
    entity::EntityRevision,
    playlist::{BriefEntity, Playlist, PlaylistWithEntriesSummary},
};

use aoide_repo::{Pagination, PaginationLimit, PaginationOffset};

use warp::http::StatusCode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum PlaylistEntriesPatchOperation {
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
    operations: impl IntoIterator<Item = PlaylistEntriesPatchOperation>,
) -> RepoResult<(EntityRevisionUpdateOutcome, Option<Playlist>)> {
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
                    return Ok((EntityRevisionUpdateOutcome::Current(hdr.rev), Some(playlist)));
                }
            }
            let mut modified = false;
            for operation in operations.into_iter() {
                use PlaylistEntriesPatchOperation::*;
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
                return Ok((EntityRevisionUpdateOutcome::Current(hdr.rev), Some(playlist)));
            }
            let updated_body_data = json::serialize_entity_body_data(&playlist.clone().into())?;
            let updated_entity = Entity::new(hdr, playlist);
            repo.update_playlist(&updated_entity, updated_body_data)
                .map(|res| (res, Some(updated_entity.body)))
        } else {
            Ok((EntityRevisionUpdateOutcome::NotFound, None))
        }
    })
}

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<bool>,

    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    // Flattening of Pagination does not work as expected:
    // https://github.com/serde-rs/serde/issues/1183
    // Workaround: Inline all parameters manually
    //#[serde(flatten)]
    //pub pagination: Pagination,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<PaginationOffset>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<PaginationLimit>,
}

impl From<PlaylistQueryParams> for (Option<bool>, Option<String>, Pagination) {
    fn from(from: PlaylistQueryParams) -> Self {
        let PlaylistQueryParams {
            brief,
            kind,
            offset,
            limit,
        } = from;
        let pagination = Pagination { offset, limit };
        (brief, kind, pagination)
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistPatchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<EntityRevision>,

    pub ops: Vec<PlaylistEntriesPatchOperation>,
}

#[allow(missing_debug_implementations)]
pub struct PlaylistsHandler {
    db: SqlitePooledConnection,
}

impl PlaylistsHandler {
    pub fn new(db: SqlitePooledConnection) -> Self {
        Self { db }
    }

    pub fn handle_create(
        &self,
        new_playlist: Playlist,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        let body_data = write_json_body_data(&new_playlist).map_err(reject_from_anyhow)?;
        let entity = create_playlist(&self.db, new_playlist.into(), body_data)
            .map_err(reject_from_anyhow)?;
        Ok(warp::reply::with_status(
            warp::reply::json(&BriefEntity::from(entity)),
            StatusCode::CREATED,
        ))
    }

    pub fn handle_update(
        &self,
        uid: EntityUid,
        entity: _serde::Entity<Playlist>,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        let json_data = write_json_body_data(&entity.1).map_err(reject_from_anyhow)?;
        let entity = Entity::from(entity);
        if uid != entity.hdr.uid {
            return Err(reject_from_anyhow(anyhow!(
                "Mismatching UIDs: {} <> {}",
                uid,
                entity.hdr.uid,
            )));
        }
        let update_result =
            update_playlist(&self.db, &entity, json_data).map_err(reject_from_anyhow)?;
        if let EntityRevisionUpdateOutcome::Updated(_, next_rev) = update_result {
            let next_hdr = EntityHeader { uid, rev: next_rev };
            let entity = Entity::new(next_hdr, entity.body);
            Ok(warp::reply::json(&BriefEntity::from(entity)))
        } else {
            Err(reject_from_anyhow(anyhow!(
                "Entity not found or revision conflict"
            )))
        }
    }

    pub fn handle_patch(
        &self,
        uid: EntityUid,
        params: PlaylistPatchParams,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        let update_result = patch_playlist(&self.db, &uid, params.rev.map(Into::into), params.ops)
            .map_err(reject_from_anyhow)?;
        use EntityRevisionUpdateOutcome::*;
        match update_result {
            (NotFound, None) => Err(reject_from_anyhow(anyhow!("Entity not found"))),
            (Current(rev), Some(body)) => {
                let hdr = EntityHeader { uid, rev };
                let entity = Entity::new(hdr, body);
                Ok(warp::reply::json(&BriefEntity::from(entity)))
            }
            (Updated(_, next_rev), Some(updated_body)) => {
                let next_hdr = EntityHeader { uid, rev: next_rev };
                let updated_entity = Entity::new(next_hdr, updated_body);
                Ok(warp::reply::json(&BriefEntity::from(updated_entity)))
            }
            _ => unreachable!("unexpected result when patching a playlist"),
        }
    }

    pub fn handle_delete(
        &self,
        uid: EntityUid,
    ) -> Result<impl warp::Reply, warp::reject::Rejection> {
        delete_playlist(&self.db, &uid)
            .map_err(reject_from_anyhow)
            .map(|res| {
                warp::reply::with_status(
                    warp::reply(),
                    res.map(|()| StatusCode::NO_CONTENT)
                        .unwrap_or(StatusCode::NOT_FOUND),
                )
            })
    }

    pub fn handle_load(&self, uid: EntityUid) -> Result<impl warp::Reply, warp::reject::Rejection> {
        load_playlist(&self.db, &uid)
            .map_err(reject_from_anyhow)
            .and_then(|res| match res {
                Some(entity_data) => {
                    let json_data =
                        json::load_entity_data_blob(&entity_data).map_err(reject_from_anyhow)?;
                    Ok(json::reply_with_content_type(json_data))
                }
                None => Err(warp::reject::not_found()),
            })
    }

    pub fn handle_list(
        &self,
        params: PlaylistQueryParams,
    ) -> Result<Box<dyn warp::Reply>, warp::reject::Rejection> {
        let (brief, kind, pagination) = params.into();
        if let Some(true) = brief {
            list_playlist_briefs(&self.db, kind.as_deref(), pagination)
                .map(|res| {
                    let res: Vec<_> = res
                        .into_iter()
                        .map(|(hdr, brief)| {
                            let hdr = _serde::EntityHeader::from(hdr);
                            let brief = PlaylistWithEntriesSummary::from(brief);
                            _serde::Entity(hdr, brief)
                        })
                        .collect();
                    warp::reply::json(&res)
                })
                .map(|reply| Box::new(reply) as Box<dyn warp::Reply>)
        } else {
            list_playlists(&self.db, kind.as_deref(), pagination)
                .and_then(|x| json::load_entity_data_array_blob(x.into_iter()))
                .map(json::reply_with_content_type)
                .map(|reply| Box::new(reply) as Box<dyn warp::Reply>)
        }
        .map_err(reject_from_anyhow)
    }
}
