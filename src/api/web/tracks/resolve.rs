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

mod uc {
    pub use crate::usecases::tracks::resolve::*;

    pub use aoide_repo::prelude::StringPredicate;
}

pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
};

///////////////////////////////////////////////////////////////////////

pub type RequestBody = Vec<String>;

pub type ResponseBody = Vec<(String, EntityHeader)>;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    uc::resolve_by_media_source_paths(
        &pooled_connection,
        collection_uid,
        request_body.into_iter().map(Into::into).collect(),
    )
    .map(|v| {
        v.into_iter()
            .map(|(path, hdr)| (path, hdr.into()))
            .collect()
    })
    .map_err(Into::into)
}
