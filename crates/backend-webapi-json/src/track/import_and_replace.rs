// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::atomic::AtomicBool;

use aoide_backend_embedded::media::predefined_faceted_tag_mapping_config;
use aoide_core_api_json::media::{
    SyncMode,
    tracker::{Completion, import_files::ImportedSourceWithIssues},
};
use aoide_core_json::track::{Entity, Track};
use aoide_media_file::io::import::{ImportTrackConfig, ImportTrackFlags};

use super::{replace::ReplaceMode, *};

mod uc {
    pub(super) use aoide_core_api::track::replace::Summary;
    pub(super) use aoide_usecases::track::import_and_replace::{Outcome, Params};
    pub(super) use aoide_usecases_sqlite::track::import_and_replace::import_and_replace_many_by_local_file_path;
}

#[derive(Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<String>,
    pub skipped: Vec<String>,
    pub failed: Vec<String>,
    pub not_imported: Vec<String>,
    pub not_created: Vec<Track>,
    pub not_updated: Vec<Track>,
}

impl From<uc::Summary> for Summary {
    fn from(from: uc::Summary) -> Self {
        let uc::Summary {
            created,
            updated,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        } = from;
        Self {
            created: created.into_iter().map(Into::into).collect(),
            updated: updated.into_iter().map(Into::into).collect(),
            unchanged: unchanged.into_iter().map(Into::into).collect(),
            skipped: skipped.into_iter().map(Into::into).collect(),
            failed: failed.into_iter().map(Into::into).collect(),
            not_imported: not_imported.into_iter().map(Into::into).collect(),
            not_created: not_created.into_iter().map(Into::into).collect(),
            not_updated: not_updated.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
    pub imported_media_sources_with_issues: Vec<ImportedSourceWithIssues>,
}

impl From<uc::Outcome> for Outcome {
    fn from(from: uc::Outcome) -> Self {
        let uc::Outcome {
            completion,
            summary,
            visited_media_source_ids: _,
            imported_media_sources_with_issues,
        } = from;
        let imported_media_sources_with_issues = imported_media_sources_with_issues
            .into_iter()
            .map(|(_, source_path, issues)| ImportedSourceWithIssues {
                path: source_path.into(),
                messages: issues.into_messages(),
            })
            .collect();
        Self {
            completion: completion.into(),
            summary: summary.into(),
            imported_media_sources_with_issues,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct QueryParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_mode: Option<SyncMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub replace_mode: Option<ReplaceMode>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode_gigtags: Option<bool>,
}

pub type RequestBody = Vec<String>;

pub type ResponseBody = Outcome;

#[tracing::instrument(
    name = "Importing and replacing tracks",
    skip(
        connection,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    connection: &mut DbConnection,
    collection_uid: &CollectionUid,
    query_params: QueryParams,
    request_body: RequestBody,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let QueryParams {
        sync_mode,
        replace_mode,
        decode_gigtags,
    } = query_params;
    let sync_mode = sync_mode.unwrap_or(SyncMode::Modified);
    let replace_mode = replace_mode.unwrap_or(ReplaceMode::UpdateOrCreate);
    // FIXME: Replace hard-coded tag mapping config
    let faceted_tag_mapping_config = predefined_faceted_tag_mapping_config();
    let mut import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config,
        ..Default::default()
    };
    if let Some(decode_gigtags) = decode_gigtags {
        import_config.flags.set(
            ImportTrackFlags::GIGTAGS_CGRP | ImportTrackFlags::GIGTAGS_COMM,
            decode_gigtags,
        );
    }
    let params = uc::Params {
        sync_mode: sync_mode.into(),
        import_config,
        replace_mode: replace_mode.into(),
    };
    let expected_content_path_count = request_body.len();
    connection
        .transaction::<_, Error, _>(|connection| {
            uc::import_and_replace_many_by_local_file_path(
                connection,
                collection_uid,
                request_body.into_iter().map(Into::into),
                Some(expected_content_path_count),
                &params,
                &std::convert::identity,
                abort_flag,
            )
            .map_err(Into::into)
        })
        .map(Into::into)
}
