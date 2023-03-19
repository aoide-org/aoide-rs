// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(feature = "backend")]
use aoide_core::util::url::{BaseUrl, BaseUrlError};
use url::Url;

use super::Completion;
use crate::{media::SyncMode, prelude::*};

mod _inner {
    pub(super) use aoide_core_api::media::tracker::import_files::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    pub sync_mode: SyncMode,
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
            sync_mode: sync_mode.into(),
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
            sync_mode: sync_mode.into(),
        })
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
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
