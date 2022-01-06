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

use aoide_core::entity::EntityUid;

use aoide_media::{
    io::export::{ExportTrackConfig, ExportTrackFlags},
    resolver::VirtualFilePathResolver,
};

use aoide_core_json::util::clock::DateTime;

use crate::media::predefined_faceted_tag_mapping_config;

use super::*;

mod uc {
    pub use aoide_usecases_sqlite::track::export_metadata::*;
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_synchronized: Option<bool>,
}

pub type ResponseBody = Option<DateTime>;

pub fn handle_request(
    connection: &SqliteConnection,
    track_uid: &EntityUid,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams { mark_synchronized } = query_params;
    let update_source_synchronized_at = mark_synchronized.unwrap_or(false);
    // FIXME: Replace hard-coded tag mapping
    let faceted_tag_mapping = predefined_faceted_tag_mapping_config();
    // FIXME: Replace hard-coded export flags
    let flags = ExportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK
        | ExportTrackFlags::AOIDE_TAGS
        | ExportTrackFlags::SERATO_MARKERS;
    let config = ExportTrackConfig {
        faceted_tag_mapping,
        flags,
    };
    let path_resolver = VirtualFilePathResolver::new();
    uc::export_metadata_into_file(
        connection,
        track_uid,
        &path_resolver,
        &config,
        update_source_synchronized_at,
    )
    .map(|ok| ok.map(Into::into))
    .map_err(Into::into)
}
