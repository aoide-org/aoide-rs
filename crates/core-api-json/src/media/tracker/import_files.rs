// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

#[cfg(feature = "backend")]
use aoide_core::util::url::{BaseUrl, BaseUrlError};

use crate::{media::SyncMode, prelude::*};

use super::Completion;

mod _inner {
    pub use aoide_core_api::media::tracker::import_files::*;
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_mode: Option<SyncMode>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Params> for Params {
    fn from(from: _inner::Params) -> Self {
        let _inner::Params {
            root_url,
            sync_mode,
        } = from;
        Self {
            root_url: root_url.map(Into::into),
            sync_mode: sync_mode.map(Into::into),
        }
    }
}

#[cfg(feature = "backend")]
impl TryFrom<Params> for _inner::Params {
    type Error = BaseUrlError;

    fn try_from(from: Params) -> Result<Self, Self::Error> {
        let Params {
            root_url,
            sync_mode,
        } = from;
        let root_url = root_url.map(BaseUrl::try_autocomplete_from).transpose()?;
        Ok(Self {
            root_url,
            sync_mode: sync_mode.map(Into::into),
        })
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImportedSourceWithIssues {
    pub path: String,
    pub messages: Vec<String>,
}

#[cfg(feature = "frontend")]
impl From<ImportedSourceWithIssues> for _inner::ImportedSourceWithIssues {
    fn from(from: ImportedSourceWithIssues) -> Self {
        let ImportedSourceWithIssues { path, messages } = from;
        Self {
            path: path.into(),
            messages,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::ImportedSourceWithIssues> for ImportedSourceWithIssues {
    fn from(from: _inner::ImportedSourceWithIssues) -> Self {
        let _inner::ImportedSourceWithIssues { path, messages } = from;
        Self {
            path: path.into(),
            messages,
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub root_path: String,
    pub completion: Completion,
    pub summary: Summary,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub imported_sources_with_issues: Vec<ImportedSourceWithIssues>,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _inner::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome {
            root_url,
            root_path,
            completion,
            summary,
            imported_sources_with_issues,
        } = from;
        Ok(Self {
            root_url: root_url.try_into()?,
            root_path: root_path.into(),
            completion: completion.into(),
            summary: summary.into(),
            imported_sources_with_issues: imported_sources_with_issues
                .into_iter()
                .map(Into::into)
                .collect(),
        })
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Outcome> for Outcome {
    fn from(from: _inner::Outcome) -> Self {
        let _inner::Outcome {
            root_url,
            root_path,
            completion,
            summary,
            imported_sources_with_issues,
        } = from;
        Self {
            root_url: root_url.into(),
            root_path: root_path.into(),
            completion: completion.into(),
            summary: summary.into(),
            imported_sources_with_issues: imported_sources_with_issues
                .into_iter()
                .map(Into::into)
                .collect(),
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TrackSummary {
    pub created: usize,
    pub updated: usize,
    pub missing: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub failed: usize,
    pub not_imported: usize,
    pub not_created: usize,
    pub not_updated: usize,
}

#[cfg(feature = "frontend")]
impl From<TrackSummary> for _inner::TrackSummary {
    fn from(from: TrackSummary) -> Self {
        let TrackSummary {
            created,
            updated,
            missing,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        } = from;
        Self {
            created,
            updated,
            missing,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::TrackSummary> for TrackSummary {
    fn from(from: _inner::TrackSummary) -> Self {
        let _inner::TrackSummary {
            created,
            updated,
            missing,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        } = from;
        Self {
            created,
            updated,
            missing,
            unchanged,
            skipped,
            failed,
            not_imported,
            not_created,
            not_updated,
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DirectorySummary {
    /// Successfully imported and marked as current.
    pub confirmed: usize,

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

#[cfg(feature = "frontend")]
impl From<DirectorySummary> for _inner::DirectorySummary {
    fn from(from: DirectorySummary) -> Self {
        let DirectorySummary {
            confirmed,
            skipped,
            untracked,
        } = from;
        Self {
            confirmed,
            skipped,
            untracked,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::DirectorySummary> for DirectorySummary {
    fn from(from: _inner::DirectorySummary) -> Self {
        let _inner::DirectorySummary {
            confirmed,
            skipped,
            untracked,
        } = from;
        Self {
            confirmed,
            skipped,
            untracked,
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub tracks: TrackSummary,
    pub directories: DirectorySummary,
}

#[cfg(feature = "frontend")]
impl From<Summary> for _inner::Summary {
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

#[cfg(feature = "backend")]
impl From<_inner::Summary> for Summary {
    fn from(from: _inner::Summary) -> Self {
        let _inner::Summary {
            tracks,
            directories,
        } = from;
        Self {
            tracks: tracks.into(),
            directories: directories.into(),
        }
    }
}
