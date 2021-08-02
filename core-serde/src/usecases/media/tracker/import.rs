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

use url::Url;

use crate::{prelude::*, usecases::media::ImportMode};

use super::Completion;

mod _core {
    pub use aoide_core::usecases::media::tracker::import::*;
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_mode: Option<ImportMode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub completion: Completion,
    pub summary: Summary,
}

impl From<Outcome> for _core::Outcome {
    fn from(from: Outcome) -> Self {
        let Outcome {
            root_url,
            completion,
            summary,
        } = from;
        Self {
            root_url,
            completion: completion.into(),
            summary: summary.into(),
        }
    }
}

impl From<_core::Outcome> for Outcome {
    fn from(from: _core::Outcome) -> Self {
        let _core::Outcome {
            root_url,
            completion,
            summary,
        } = from;
        Self {
            root_url,
            completion: completion.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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

impl From<TrackSummary> for _core::TrackSummary {
    fn from(from: TrackSummary) -> Self {
        let TrackSummary {
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

impl From<_core::TrackSummary> for TrackSummary {
    fn from(from: _core::TrackSummary) -> Self {
        let _core::TrackSummary {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

    /// Untracked directories that have not been found.
    ///
    /// Directories that are not found during the import are
    /// untracked implicitly.
    pub untracked: usize,
}

impl From<DirectorySummary> for _core::DirectorySummary {
    fn from(from: DirectorySummary) -> Self {
        let DirectorySummary {
            confirmed,
            rejected,
            skipped,
            untracked,
        } = from;
        Self {
            confirmed,
            rejected,
            skipped,
            untracked,
        }
    }
}

impl From<_core::DirectorySummary> for DirectorySummary {
    fn from(from: _core::DirectorySummary) -> Self {
        let _core::DirectorySummary {
            confirmed,
            rejected,
            skipped,
            untracked,
        } = from;
        Self {
            confirmed,
            rejected,
            skipped,
            untracked,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Summary {
    pub tracks: TrackSummary,
    pub directories: DirectorySummary,
}

impl From<Summary> for _core::Summary {
    fn from(from: Summary) -> Self {
        let Summary {
            tracks,
            directories,
        } = from;
        Self {
            tracks: tracks.into(),
            directories: directories.into(),
        }
    }
}

impl From<_core::Summary> for Summary {
    fn from(from: _core::Summary) -> Self {
        let _core::Summary {
            tracks,
            directories,
        } = from;
        Self {
            tracks: tracks.into(),
            directories: directories.into(),
        }
    }
}
