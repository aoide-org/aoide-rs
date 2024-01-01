// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::{media::content::ContentPathConfig, prelude::*, util::color::Color};

mod _core {
    pub(super) use aoide_core::collection::*;
}

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaSourceConfig {
    content_path: ContentPathConfig,
}

impl TryFrom<MediaSourceConfig> for _core::MediaSourceConfig {
    type Error = anyhow::Error;

    fn try_from(from: MediaSourceConfig) -> anyhow::Result<Self> {
        let MediaSourceConfig { content_path } = from;
        let into = Self {
            content_path: content_path.try_into()?,
        };
        Ok(into)
    }
}

impl From<_core::MediaSourceConfig> for MediaSourceConfig {
    fn from(from: _core::MediaSourceConfig) -> Self {
        let _core::MediaSourceConfig { content_path } = from;
        Self {
            content_path: content_path.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Collection {
    title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,

    media_source_config: MediaSourceConfig,
}

impl TryFrom<Collection> for _core::Collection {
    type Error = anyhow::Error;

    fn try_from(from: Collection) -> anyhow::Result<Self> {
        let Collection {
            title,
            notes,
            kind,
            color,
            media_source_config,
        } = from;
        let into = Self {
            title,
            notes,
            kind,
            color: color.map(Into::into),
            media_source_config: media_source_config.try_into()?,
        };
        Ok(into)
    }
}

impl From<_core::Collection> for Collection {
    fn from(from: _core::Collection) -> Self {
        let _core::Collection {
            title,
            notes,
            kind,
            color,
            media_source_config,
        } = from;
        Self {
            title,
            notes,
            kind,
            color: color.map(Into::into),
            media_source_config: media_source_config.into(),
        }
    }
}

///////////////////////////////////////////////////////////////////////
// Entity
///////////////////////////////////////////////////////////////////////

pub type Entity = crate::entity::Entity<Collection>;

impl TryFrom<Entity> for _core::Entity {
    type Error = anyhow::Error;

    fn try_from(from: Entity) -> anyhow::Result<Self> {
        Self::try_new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        let (hdr, body) = from.into();
        Self(hdr.into_untyped().into(), body.into())
    }
}
