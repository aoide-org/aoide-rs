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

use super::*;

mod uc {
    pub use crate::usecases::media::tracker::import::*;
}

mod _core {
    pub use aoide_core::entity::EntityUid;
}

use aoide_core::track::tag::{FACET_GENRE, FACET_MOOD};
pub use aoide_core_serde::{
    entity::EntityHeader,
    track::{Entity, Track},
};
use aoide_media::{
    io::import::{ImportTrackConfig, ImportTrackFlags},
    util::tag::{FacetedTagMappingConfigInner, TagMappingConfig},
};

use tokio::sync::watch;
use url::Url;

///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_mode: Option<ImportMode>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSummary {
    pub created: usize,
    pub updated: usize,
    pub missing: usize,
    pub unchanged: usize,
    pub not_imported: usize,
    pub not_created: usize,
    pub not_updated: usize,
}

impl From<uc::TrackSummary> for TrackSummary {
    fn from(from: uc::TrackSummary) -> Self {
        let uc::TrackSummary {
            created,
            updated,
            missing,
            unchanged,
            not_imported,
            not_created,
            not_updated,
        } = from;
        Self {
            created,
            updated,
            missing,
            unchanged,
            not_imported,
            not_created,
            not_updated,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct DirectorySummary {
    /// Successfully imported and marked as current.
    pub confirmed: usize,

    /// Rejected directories are retried repeatedly.
    ///
    /// This may only happen due to race condition if multiple
    /// concurrent tasks are running. Currently this could never
    /// happen due to an exclusive lock on the database.
    pub rejected: usize,

    /// Skipped directories will not be retried.
    ///
    /// Directories are skipped on non-recoverable errors that
    /// would occur again when retrying the import. Yet the import
    /// will be retried after restarting the import task.
    pub skipped: usize,
}

impl From<uc::DirectorySummary> for DirectorySummary {
    fn from(from: uc::DirectorySummary) -> Self {
        let uc::DirectorySummary {
            confirmed,
            rejected,
            skipped,
        } = from;
        Self {
            confirmed,
            rejected,
            skipped,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Summary {
    pub tracks: TrackSummary,
    pub directories: DirectorySummary,
}

impl From<uc::Summary> for Summary {
    fn from(from: uc::Summary) -> Self {
        let uc::Summary {
            tracks,
            directories,
        } = from;
        Self {
            tracks: tracks.into(),
            directories: directories.into(),
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
        } = from;
        Self {
            completion: completion.into(),
            summary: summary.into(),
        }
    }
}

pub type RequestBody = Params;

pub type ResponseBody = Outcome;

pub fn handle_request(
    pooled_connection: SqlitePooledConnection,
    collection_uid: &_core::EntityUid,
    request_body: RequestBody,
    progress_summary_tx: Option<&watch::Sender<uc::Summary>>,
    abort_flag: &AtomicBool,
) -> Result<ResponseBody> {
    let RequestBody {
        root_url,
        import_mode,
    } = request_body;
    let import_mode = import_mode.unwrap_or(ImportMode::Modified);
    // FIXME: Replace hard-coded tag mapping config
    let mut faceted_tag_mapping_config = FacetedTagMappingConfigInner::default();
    faceted_tag_mapping_config.insert(
        FACET_GENRE.to_owned().into(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    faceted_tag_mapping_config.insert(
        FACET_MOOD.to_owned().into(),
        TagMappingConfig {
            label_separator: ";".into(),
            split_score_attenuation: 0.75,
        },
    );
    let import_config = ImportTrackConfig {
        faceted_tag_mapping: faceted_tag_mapping_config.into(),
    };
    // FIXME: Replace hard-coded import flags
    let import_flags = ImportTrackFlags::ARTWORK_DIGEST
        | ImportTrackFlags::ITUNES_ID3V2_GROUPING_MOVEMENT_WORK
        | ImportTrackFlags::MIXXX_CUSTOM_TAGS
        | ImportTrackFlags::SERATO_TAGS;
    Ok(uc::import(
        &pooled_connection,
        collection_uid,
        root_url.as_ref(),
        import_mode.into(),
        &import_config,
        import_flags,
        progress_summary_tx,
        abort_flag,
    )
    .map(Into::into)?)
}
