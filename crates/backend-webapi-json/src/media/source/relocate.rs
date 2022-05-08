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

use aoide_core::media::content::ContentPath;

use super::*;

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RequestBody {
    old_path_prefix: String,
    new_path_prefix: String,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResponseBody {
    replaced_count: usize,
}

pub fn handle_request(
    connection: &SqliteConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody {
        old_path_prefix,
        new_path_prefix,
    } = request_body;
    connection
        .transaction::<_, Error, _>(|| {
            aoide_usecases_sqlite::media::source::relocate::relocate(
                connection,
                collection_uid,
                &ContentPath::new(old_path_prefix),
                &ContentPath::new(new_path_prefix),
            )
            .map_err(Into::into)
        })
        .map(|replaced_count| ResponseBody { replaced_count })
}
