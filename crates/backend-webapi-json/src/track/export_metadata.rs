// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::media::content::resolver::VirtualFilePathResolver;

use aoide_media::io::export::{ExportTrackConfig, ExportTrackFlags};

use crate::media::predefined_faceted_tag_mapping_config;

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

pub type ResponseBody = bool;

pub fn handle_request(
    connection: &mut DbConnection,
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
    connection.transaction::<_, Error, _>(|connection| {
        uc::export_metadata_into_file(connection, track_uid, &path_resolver, &config)
            .map_err(Into::into)
    })
}
