// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#[cfg(feature = "backend")]
use aoide_core::util::url::{BaseUrl, BaseUrlError};
use url::Url;

use super::DirTrackingStatus;
use crate::prelude::*;

mod _inner {
    pub(super) use aoide_core_api::media::tracker::untrack_directories::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DirTrackingStatus>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Params> for Params {
    fn from(from: _inner::Params) -> Self {
        let _inner::Params { root_url, status } = from;
        Self {
            root_url: root_url.map(Into::into),
            status: status.map(Into::into),
        }
    }
}

#[cfg(feature = "backend")]
impl TryFrom<Params> for _inner::Params {
    type Error = BaseUrlError;

    fn try_from(from: Params) -> Result<Self, Self::Error> {
        let Params { root_url, status } = from;
        let root_url = root_url.map(BaseUrl::try_autocomplete_from).transpose()?;
        Ok(Self {
            root_url,
            status: status.map(Into::into),
        })
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
    pub summary: Summary,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _inner::Outcome {
    type Error = anyhow::Error;

    fn try_from(from: Outcome) -> std::result::Result<Self, Self::Error> {
        let Outcome {
            root_url,
            root_path,
            summary,
        } = from;
        Ok(Self {
            root_url: root_url.try_into()?,
            root_path: root_path.into(),
            summary: summary.into(),
        })
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Outcome> for Outcome {
    fn from(from: _inner::Outcome) -> Self {
        let _inner::Outcome {
            root_url,
            root_path,
            summary,
        } = from;
        Self {
            root_url: root_url.into(),
            root_path: root_path.into(),
            summary: summary.into(),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Summary {
    pub untracked: u64,
}

#[cfg(feature = "frontend")]
#[allow(clippy::cast_possible_truncation)]
impl From<Summary> for _inner::Summary {
    fn from(from: Summary) -> Self {
        let Summary { untracked } = from;
        Self {
            untracked: untracked as usize,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::Summary> for Summary {
    fn from(from: _inner::Summary) -> Self {
        let _inner::Summary { untracked } = from;
        Self {
            untracked: untracked as u64,
        }
    }
}
