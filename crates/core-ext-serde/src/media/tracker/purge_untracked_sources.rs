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

use crate::prelude::*;

mod _inner {
    pub use aoide_core_ext::media::tracker::purge_untracked_sources::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub untrack_orphaned_directories: Option<bool>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Params> for Params {
    fn from(from: _inner::Params) -> Self {
        let _inner::Params {
            root_url,
            untrack_orphaned_directories,
        } = from;
        Self {
            root_url: root_url.map(Into::into),
            untrack_orphaned_directories,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub summary: Summary,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _inner::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome { root_url, summary } = from;
        let root_url = root_url.try_into()?;
        Ok(Self {
            root_url,
            summary: summary.into(),
        })
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Outcome> for Outcome {
    fn from(from: _inner::Outcome) -> Self {
        let _inner::Outcome { root_url, summary } = from;
        Self {
            root_url: root_url.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    untracked_directories: u64,
    purged_media_sources: u64,
}

#[cfg(feature = "frontend")]
impl From<Summary> for _inner::Summary {
    fn from(from: Summary) -> Self {
        let Summary {
            untracked_directories,
            purged_media_sources,
        } = from;
        Self {
            untracked_directories: untracked_directories as usize,
            purged_media_sources: purged_media_sources as usize,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Summary> for Summary {
    fn from(from: _inner::Summary) -> Self {
        let _inner::Summary {
            untracked_directories,
            purged_media_sources,
        } = from;
        Self {
            untracked_directories: untracked_directories as u64,
            purged_media_sources: purged_media_sources as u64,
        }
    }
}
