// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_core_json::entity::EntityHeader;

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::resolve::*;
}

pub type RequestBody = Vec<String>;

pub type ResponseBody = Vec<(String, EntityHeader)>;

pub fn handle_request(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    connection
        .transaction::<_, Error, _>(|| {
            uc::resolve_by_media_source_content_paths(
                connection,
                collection_uid,
                request_body.into_iter().map(Into::into).collect(),
            )
            .map_err(Into::into)
        })
        .map(|v| {
            v.into_iter()
                .map(|(content_path, hdr)| (content_path, hdr.into()))
                .collect()
        })
}
