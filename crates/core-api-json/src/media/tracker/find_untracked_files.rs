// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

use crate::prelude::*;

use super::Completion;

pub type Params = super::FsTraversalParams;

mod _core {
    pub(super) use aoide_core_api::media::tracker::find_untracked_files::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    pub root_url: Url,
    pub root_path: String,
    pub completion: Completion,
    pub content_paths: Vec<String>,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _core::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome {
            root_url,
            root_path,
            completion,
            content_paths,
        } = from;
        Ok(Self {
            root_url: root_url.try_into()?,
            root_path: root_path.into(),
            completion: completion.into(),
            content_paths: content_paths.into_iter().map(Into::into).collect(),
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
            content_paths,
        } = from;
        Self {
            root_url: root_url.into(),
            root_path: root_path.into(),
            completion: completion.into(),
            content_paths: content_paths.into_iter().map(Into::into).collect(),
        }
    }
}
