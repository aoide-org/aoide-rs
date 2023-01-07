// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use url::Url;

#[cfg(feature = "backend")]
use aoide_core::util::url::{BaseUrl, BaseUrlError};

use crate::prelude::*;

mod _inner {
    pub(super) use aoide_core_api::media::source::purge_orphaned::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Serialize))]
#[cfg_attr(feature = "backend", derive(Deserialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Params {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,
}

#[cfg(feature = "frontend")]
impl From<_inner::Params> for Params {
    fn from(from: _inner::Params) -> Self {
        let _inner::Params { root_url } = from;
        Self {
            root_url: root_url.map(Into::into),
        }
    }
}

#[cfg(feature = "backend")]
impl TryFrom<Params> for _inner::Params {
    type Error = BaseUrlError;

    fn try_from(from: Params) -> Result<Self, Self::Error> {
        let Params { root_url } = from;
        let root_url = root_url.map(BaseUrl::try_autocomplete_from).transpose()?;
        Ok(Self { root_url })
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Outcome {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_path: Option<String>,

    pub summary: Summary,
}

#[cfg(feature = "frontend")]
impl TryFrom<Outcome> for _inner::Outcome {
    type Error = aoide_core::util::url::BaseUrlError;

    fn try_from(from: Outcome) -> Result<Self, Self::Error> {
        let Outcome {
            root_url,
            root_path,
            summary,
        } = from;
        Ok(Self {
            root_url: root_url.map(TryInto::try_into).transpose()?,
            root_path: root_path.map(Into::into),
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
            root_url: root_url.map(Into::into),
            root_path: root_path.map(Into::into),
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
    pub purged: u64,
}

#[cfg(feature = "frontend")]
#[allow(clippy::cast_possible_truncation)]
impl From<Summary> for _inner::Summary {
    fn from(from: Summary) -> Self {
        let Summary { purged } = from;
        Self {
            purged: purged as usize,
        }
    }
}

#[cfg(feature = "backend")]
#[allow(clippy::cast_possible_truncation)]
impl From<_inner::Summary> for Summary {
    fn from(from: _inner::Summary) -> Self {
        let _inner::Summary { purged } = from;
        Self {
            purged: purged as u64,
        }
    }
}
