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

use aoide_usecases_sqlite::SqlitePooledConnection;

use aoide_core::util::url::BaseUrl;

use super::*;

mod uc {
    pub use aoide_repo::prelude::StringPredicate;
    pub use aoide_usecases_sqlite::track::purge::*;
}

pub type RequestBody = aoide_core_ext_serde::track::purge_untracked::Params;

pub type ResponseBody = aoide_core_ext_serde::track::purge_untracked::Outcome;

#[tracing::instrument(
    name = "Purging untracked tracks",
    skip(
        pooled_connection,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        untrack_orphaned_directories,
    } = request_body;
    let root_url = root_url
        .map(BaseUrl::try_autocomplete_from)
        .transpose()
        .map_err(anyhow::Error::from)
        .map_err(Error::BadRequest)?;
    let params = aoide_core_ext::track::purge_untracked::Params {
        root_url,
        untrack_orphaned_directories,
    };
    uc::purge_by_untracked_media_sources(&pooled_connection, collection_uid, &params)
        .map(Into::into)
        .map_err(Into::into)
}
