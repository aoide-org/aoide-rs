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

use crate::{media::ImportMode, prelude::*};

use super::Completion;

mod _inner {
    pub use aoide_core_ext::media::tracker::import::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub import_mode: Option<ImportMode>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Params> for Params {
    fn from(from: _inner::Params) -> Self {
        let _inner::Params {
            root_url,
            import_mode,
        } = from;
        Self {
            root_url: root_url.map(Into::into),
            import_mode: import_mode.map(Into::into),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub completion: Completion,
    pub summary: Summary,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _inner::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome {
            root_url,
            completion,
            summary,
        } = from;
        let root_url = root_url.try_into()?;
        Ok(Self {
            root_url,
            completion: completion.into(),
            summary: summary.into(),
        })
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Outcome> for Outcome {
    fn from(from: _inner::Outcome) -> Self {
        let _inner::Outcome {
            root_url,
            completion,
            summary,
        } = from;
        Self {
            root_url: root_url.into(),
            completion: completion.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
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

#[cfg(feature = "frontend")]
impl From<TrackSummary> for _inner::TrackSummary {
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

#[cfg(feature = "backend")]
impl From<_inner::TrackSummary> for TrackSummary {
    fn from(from: _inner::TrackSummary) -> Self {
        let _inner::TrackSummary {
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

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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

#[cfg(feature = "frontend")]
impl From<DirectorySummary> for _inner::DirectorySummary {
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

#[cfg(feature = "backend")]
impl From<_inner::DirectorySummary> for DirectorySummary {
    fn from(from: _inner::DirectorySummary) -> Self {
        let _inner::DirectorySummary {
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

#[derive(Debug)]
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
