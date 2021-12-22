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

use std::sync::atomic::AtomicBool;

use aoide_core::{entity::EntityUid, util::url::BaseUrl};
use aoide_usecases_sqlite::SqlitePooledConnection;

use super::*;

mod uc {
    pub use aoide_usecases::media::tracker::find_untracked_files::ProgressEvent;
    pub use aoide_usecases_sqlite::media::tracker::find_untracked_files::*;
}

pub type RequestBody = aoide_core_api_json::media::tracker::FsTraversalParams;

pub type ResponseBody = aoide_core_api_json::media::tracker::find_untracked_files::Outcome;

#[tracing::instrument(
    name = "Finding untracked media sources",
    skip(
        pooled_connection,
        progress_event_fn,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    request_body: RequestBody,
    progress_event_fn: &mut impl FnMut(uc::ProgressEvent),
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        max_depth,
    } = request_body;
    let root_url = root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?
        .map(Into::into);
    let params = aoide_core_api::media::tracker::FsTraversalParams {
        root_url,
        max_depth,
    };
    uc::visit_directories(
        &pooled_connection,
        collection_uid,
        &params,
        progress_event_fn,
        abort_flag,
    )
    .map(Into::into)
    .map_err(Into::into)
}
