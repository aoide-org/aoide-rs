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

use super::Completion;

mod _core {
    pub use aoide_core_ext::media::tracker::scan::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub current: usize,
    pub added: usize,
    pub modified: usize,
    pub orphaned: usize,
    pub skipped: usize,
}

#[cfg(feature = "frontend")]
impl From<Summary> for _core::Summary {
    fn from(from: Summary) -> Self {
        let Summary {
            current,
            added,
            modified,
            orphaned,
            skipped,
        } = from;
        Self {
            current,
            added,
            modified,
            orphaned,
            skipped,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_core::Summary> for Summary {
    fn from(from: _core::Summary) -> Self {
        let _core::Summary {
            current,
            added,
            modified,
            orphaned,
            skipped,
        } = from;
        Self {
            current,
            added,
            modified,
            orphaned,
            skipped,
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub completion: Completion,
    pub summary: Summary,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _core::Outcome {
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
impl From<_core::Outcome> for Outcome {
    fn from(from: _core::Outcome) -> Self {
        let _core::Outcome {
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
