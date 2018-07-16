// aoide.org - Copyright (C) 2018 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use actix::prelude::*;

use actix_web::*;

use diesel::prelude::*;

use failure::{Error, Fail};

use futures::future::Future;

use aoide_core::domain::entity::*;

use aoide_storage::{
    api::{
        album::{AlbumSummary, Albums, AlbumsResult}, Pagination,
    },
    storage::track::TrackRepository,
};

use super::{AppState, SqliteExecutor};

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumsQueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_uid: Option<EntityUid>,
}

#[derive(Debug, Default)]
struct ListAlbumsMessage {
    pub collection_uid: Option<EntityUid>,
    pub pagination: Pagination,
}

pub type ListAlbumsResult = AlbumsResult<Vec<AlbumSummary>>;

impl Message for ListAlbumsMessage {
    type Result = ListAlbumsResult;
}

impl Handler<ListAlbumsMessage> for SqliteExecutor {
    type Result = ListAlbumsResult;

    fn handle(&mut self, msg: ListAlbumsMessage, _: &mut Self::Context) -> Self::Result {
        let connection = &*self.pooled_connection()?;
        let repository = TrackRepository::new(connection);
        connection.transaction::<_, Error, _>(|| {
            repository.list_albums(msg.collection_uid.as_ref(), msg.pagination)
        })
    }
}

pub fn on_list_albums(
    (state, query_tracks, query_pagination): (
        State<AppState>,
        Query<AlbumsQueryParams>,
        Query<Pagination>,
    ),
) -> FutureResponse<HttpResponse> {
    let msg = ListAlbumsMessage {
        collection_uid: query_tracks.into_inner().collection_uid,
        pagination: query_pagination.into_inner(),
    };
    state
        .executor
        .send(msg)
        .map_err(|err| err.compat())
        .from_err()
        .and_then(|res| match res {
            Ok(tags) => Ok(HttpResponse::Ok().json(tags)),
            Err(e) => Err(e.into()),
        })
        .responder()
}
