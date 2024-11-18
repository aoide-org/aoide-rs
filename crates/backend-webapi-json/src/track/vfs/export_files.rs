// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::Path;

use serde::{Deserialize, Serialize};

use aoide_core::CollectionUid;
use aoide_core_api_json::track::search::Filter;
use aoide_repo_sqlite::DbConnection;
use aoide_usecases::track::vfs::export_files::ExportTrackFilesOutcome;
use aoide_usecases_sqlite::track::vfs::export_files;

use crate::Result;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct RequestBody {
    target_root_path: String,
    filter: Option<Filter>,
    batch_size: Option<u64>,
    purge_other_files: Option<bool>,
}

#[derive(Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct ResponseBody {
    exported: u64,
    skipped: u64,
    failed: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    purged: Option<u64>,
}

#[allow(clippy::panic_in_result_fn)] // tracing::instrument
pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    request_body: RequestBody,
) -> Result<ResponseBody> {
    let RequestBody {
        target_root_path,
        filter,
        batch_size,
        purge_other_files,
    } = request_body;
    let filter = filter.map(Into::into);
    let target_root_path = Path::new(&target_root_path);
    let purge_other_files = purge_other_files.unwrap_or(false);
    let outcome = export_files(
        connection,
        collection_uid,
        filter.as_ref(),
        batch_size,
        target_root_path,
        Default::default(),
        purge_other_files,
    )?;
    let ExportTrackFilesOutcome {
        exported,
        skipped,
        failed,
        purged,
    } = outcome;
    let failed_count = failed.len();
    if failed_count > 0 {
        // TODO: Return detailed errors in response.
        log::warn!("Failed to export {failed_count} track file(s): {failed:?}");
    }
    let response_body = ResponseBody {
        exported,
        skipped,
        purged,
        failed: failed_count as _,
    };
    Ok(response_body)
}
