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

use super::DirTrackingStatus;

mod _core {
    pub use aoide_core_ext::media::tracker::{untrack::*, DirTrackingStatus};
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    pub root_url: Url,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DirTrackingStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub untracked: usize,
}

impl From<Summary> for _core::Summary {
    fn from(from: Summary) -> Self {
        let Summary { untracked } = from;
        Self { untracked }
    }
}

impl From<_core::Summary> for Summary {
    fn from(from: _core::Summary) -> Self {
        let _core::Summary { untracked } = from;
        Self { untracked }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub summary: Summary,
}

impl From<Outcome> for _core::Outcome {
    fn from(from: Outcome) -> Self {
        let Outcome { root_url, summary } = from;
        Self {
            root_url,
            summary: summary.into(),
        }
    }
}

impl From<_core::Outcome> for Outcome {
    fn from(from: _core::Outcome) -> Self {
        let _core::Outcome { root_url, summary } = from;
        Self {
            root_url,
            summary: summary.into(),
        }
    }
}
