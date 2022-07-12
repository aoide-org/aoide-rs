// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use crate::prelude::*;

use super::Completion;

pub type Params = super::FsTraversalParams;

mod _core {
    pub(super) use aoide_core_api::media::tracker::scan_directories::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub root_path: String,
    pub completion: Completion,
    pub summary: Summary,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _core::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome {
            root_url,
            root_path,
            completion,
            summary,
        } = from;
        Ok(Self {
            root_url: root_url.try_into()?,
            root_path: root_path.into(),
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
            root_path,
            completion,
            summary,
        } = from;
        Self {
            root_url: root_url.into(),
            root_path: root_path.into(),
            completion: completion.into(),
            summary: summary.into(),
        }
    }
}
