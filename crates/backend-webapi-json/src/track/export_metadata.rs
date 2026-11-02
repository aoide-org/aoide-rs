// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_backend_embedded::media::{
    predefined_export_track_config, predefined_import_track_config,
};
use aoide_repo::RepoError;
use aoide_usecases::track::update::{ExportParams, Outcome};

use super::*;

mod uc {
    pub(super) use aoide_usecases_sqlite::track::{load::load_one, update::export_one};
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(
    feature = "json-schema",
    derive(schemars::JsonSchema),
    schemars(rename = "ExportTrackMetadataQueryParams")
)]
pub struct QueryParams {
    // TODO: Add export options
}

pub type ResponseBody = ();

/// Write the metadata of a track into its file and re-import it.
///
/// Fails with a conflict if the file has been modified on disk since it
/// has been imported for the last time.
pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    track_uid: &EntityUid,
    _query_params: QueryParams,
) -> Result<ResponseBody> {
    // FIXME: Replace hard-coded tag mapping config
    let export = ExportParams {
        export_config: predefined_export_track_config(),
        import_config: predefined_import_track_config(),
    };
    connection.transaction::<_, Error, _>(|connection| {
        let expected_entity_hdr = uc::load_one(connection, track_uid)?.raw.hdr;
        match uc::export_one(connection, collection_uid, &expected_entity_hdr, export)? {
            Outcome::Updated { .. } => Ok(()),
            Outcome::Rejected(conflict) => {
                log::info!("Rejected export of track {track_uid}: {conflict:?}");
                Err(aoide_usecases::Error::from(RepoError::Conflict).into())
            }
        }
    })
}
