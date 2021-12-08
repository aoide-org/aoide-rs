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

use std::sync::atomic::AtomicBool;

use aoide_media::io::import::{ImportTrackConfig, ImportTrackFlags};

use aoide_core_ext_serde::media::SyncMode;

use aoide_core_serde::track::{Entity, Track};
use aoide_usecases_sqlite::SqlitePooledConnection;

use crate::media::predefined_faceted_tag_mapping_config;

use super::{replace::ReplaceMode, *};

mod uc {
    pub use aoide_core_ext::track::replace::Summary;
    pub use aoide_usecases::track::replace::{Completion, Outcome};
    pub use aoide_usecases_sqlite::track::replace::*;
}

mod _inner {
    pub use aoide_core::entity::EntityUid;
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub created: Vec<Entity>,
    pub updated: Vec<Entity>,
    pub unchanged: Vec<String>,
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
            not_imported,
            not_created,
            not_updated,
        } = from;
        Self {
            created: created.into_iter().map(Into::into).collect(),
            updated: updated.into_iter().map(Into::into).collect(),
            unchanged: unchanged.into_iter().map(Into::into).collect(),
            not_imported: not_imported.into_iter().map(Into::into).collect(),
            not_created: not_created.into_iter().map(Into::into).collect(),
            not_updated: not_updated.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Completion {
    Finished,
    Aborted,
}

impl From<uc::Completion> for Completion {
    fn from(from: uc::Completion) -> Self {
        use uc::Completion::*;
        match from {
            Finished => Self::Finished,
            Aborted => Self::Aborted,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub completion: Completion,
    pub summary: Summary,
}

impl From<uc::Outcome> for Outcome {
    fn from(from: uc::Outcome) -> Self {
        let uc::Outcome {
            completion,
            summary,
            media_source_ids: _,
        } = from;
        Self {
            completion: completion.into(),
            summary: summary.into(),
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
}

pub type RequestBody = Vec<String>;

pub type ResponseBody = Outcome;

#[tracing::instrument(
    name = "Importing and replacing tracks",
    skip(
        pooled_connection,
        abort_flag,
    ),
    fields(
        request_id = %new_request_id(),
    )
)]
pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_inner::EntityUid,
    query_params: QueryParams,
    request_body: RequestBody,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let QueryParams {
        sync_mode,
        replace_mode,
    } = query_params;
    let sync_mode = sync_mode.unwrap_or(SyncMode::Modified);
    let replace_mode = replace_mode.unwrap_or(ReplaceMode::UpdateOrCreate);
    // FIXME: Replace hard-coded tag mapping config
    let faceted_tag_mapping_config = predefined_faceted_tag_mapping_config();
    // FIXME: Replace hard-coded import flags
    let import_flags = ImportTrackFlags::ARTWORK_DIGEST
        | ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK
        | ImportTrackFlags::AOIDE_TAGS
        | ImportTrackFlags::SERATO_MARKERS;
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config,
        flags: import_flags,
    };
    let expected_source_path_count = request_body.len();
    uc::import_and_replace_by_local_file_path_iter(
        &pooled_connection,
        collection_uid,
        sync_mode.into(),
        &import_config,
        replace_mode.into(),
        request_body.into_iter().map(Into::into),
        Some(expected_source_path_count),
        abort_flag,
    )
    .map(Into::into)
    .map_err(Into::into)
}
