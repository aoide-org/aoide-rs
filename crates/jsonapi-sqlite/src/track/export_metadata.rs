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

use aoide_core::entity::EntityUid;

use aoide_media::{
    io::export::{ExportTrackConfig, ExportTrackFlags},
    resolver::VirtualFilePathResolver,
};

use aoide_core_serde::util::clock::DateTime;

use crate::media::predefined_faceted_tag_mapping_config;

use super::*;

mod uc {
    pub use aoide_usecases_sqlite::track::export_metadata::*;
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark_synchronized: Option<bool>,
}

pub type ResponseBody = Option<DateTime>;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &EntityUid,
    track_uid: &EntityUid,
    query_params: QueryParams,
) -> Result<ResponseBody> {
    let QueryParams { mark_synchronized } = query_params;
    let update_source_synchronized_at = mark_synchronized.unwrap_or(false);
    let config = ExportTrackConfig {
        faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
        flags: ExportTrackFlags::all(),
    };
    let path_resolver = VirtualFilePathResolver::new();
    let media_source_synchronized_at = uc::export_metadata_into_file(
        &pooled_connection,
        collection_uid,
        track_uid,
        &path_resolver,
        &config,
        update_source_synchronized_at,
    )?;
    Ok(media_source_synchronized_at.map(Into::into))
}
