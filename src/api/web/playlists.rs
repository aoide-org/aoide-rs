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

mod json {
    pub use super::super::json::*;
    pub use crate::usecases::playlists::json::*;
}

use crate::usecases::playlists::*;

mod _serde {
    pub use aoide_core_serde::entity::{Entity, EntityHeader};
}

use aoide_core::{
    entity::{EntityHeader, EntityRevisionUpdateResult, EntityUid},
    playlist::Entity,
};

use aoide_core_serde::{
    entity::EntityRevision,
    playlist::{Playlist, PlaylistBrief, BriefEntity},
};

use aoide_repo::{Pagination, PaginationLimit, PaginationOffset};

use warp::http::StatusCode;

///////////////////////////////////////////////////////////////////////

#[derive(Debug, Default, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub brief: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

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
            r#type,
            offset,
            limit,
        } = from;
        let pagination = Pagination { offset, limit };
        (brief, r#type, pagination)
    }
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct PlaylistPatchParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<EntityRevision>,

    pub ops: Vec<PlaylistPatchOperation>,
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
        if let EntityRevisionUpdateResult::Updated(_, next_rev) = update_result {
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
        use EntityRevisionUpdateResult::*;
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
                        json::load_entity_data_blob(entity_data).map_err(reject_from_anyhow)?;
                    Ok(json::reply_with_content_type(json_data))
                }
                None => Err(warp::reject::not_found()),
            })
    }

    pub fn handle_list(
        &self,
        params: PlaylistQueryParams,
    ) -> Result<Box<dyn warp::Reply>, warp::reject::Rejection> {
        let (brief, r#type, pagination) = params.into();
        if let Some(true) = brief {
            list_playlist_briefs(&self.db, r#type.as_ref().map(String::as_str), pagination)
                .map(|res| {
                    let res: Vec<_> = res
                        .into_iter()
                        .map(|(hdr, brief)| {
                            let hdr = _serde::EntityHeader::from(hdr);
                            let brief = PlaylistBrief::from(brief);
                            _serde::Entity(hdr, brief)
                        })
                        .collect();
                    warp::reply::json(&res)
                })
                .map(|reply| Box::new(reply) as Box<dyn warp::Reply>)
        } else {
            list_playlists(&self.db, r#type.as_ref().map(String::as_str), pagination)
                .and_then(|x| json::load_entity_data_array_blob(x.into_iter()))
                .map(json::reply_with_content_type)
                .map(|reply| Box::new(reply) as Box<dyn warp::Reply>)
        }
        .map_err(reject_from_anyhow)
    }
}
