// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_backend_embedded::media::predefined_faceted_tag_mapping_config;
use aoide_core::media::content::resolver::vfs::VfsResolver;
use aoide_media_file::io::export::ExportTrackConfig;

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::export_metadata::*;
}

#[derive(Debug, Deserialize)]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    // TODO: Add export options
}

pub type ResponseBody = ();

#[allow(clippy::needless_pass_by_value)] // consume arguments
pub fn handle_request(
    connection: &mut DbConnection,
    track_uid: &EntityUid,
    _query_params: QueryParams,
) -> Result<ResponseBody> {
    // FIXME: Replace hard-coded tag mapping config
    let faceted_tag_mapping = predefined_faceted_tag_mapping_config();
    let config = ExportTrackConfig {
        faceted_tag_mapping,
        ..Default::default()
    };
    let path_resolver = VfsResolver::new();
    // TODO: Support editing the embedded artwork image(s) (optional)
    let edit_embedded_artwork_image = None;
    connection.transaction::<_, Error, _>(|connection| {
        uc::export_metadata_into_file(
            connection,
            track_uid,
            &path_resolver,
            &config,
            edit_embedded_artwork_image,
        )
        .map_err(Into::into)
    })
}
