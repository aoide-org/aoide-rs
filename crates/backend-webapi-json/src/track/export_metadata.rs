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

use aoide_core::media::content::resolver::VirtualFilePathResolver;

use aoide_media::io::export::{ExportTrackConfig, ExportTrackFlags};

use crate::media::predefined_faceted_tag_mapping_config;

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::export_metadata::*;
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    // TODO: Add export options
}

pub type ResponseBody = bool;

pub fn handle_request(
    connection: &SqliteConnection,
    track_uid: &EntityUid,
    _query_params: QueryParams,
) -> Result<ResponseBody> {
    // FIXME: Replace hard-coded tag mapping
    let faceted_tag_mapping = predefined_faceted_tag_mapping_config();
    // FIXME: Replace hard-coded export flags
    let flags = ExportTrackFlags::all();
    let config = ExportTrackConfig {
        faceted_tag_mapping,
        flags,
    };
    let path_resolver = VirtualFilePathResolver::new();
    connection.transaction::<_, Error, _>(|| {
        uc::export_metadata_into_file(connection, track_uid, &path_resolver, &config)
            .map_err(Into::into)
    })
}
