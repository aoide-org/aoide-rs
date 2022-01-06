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

use crate::{media::SourcePathConfig, prelude::*, util::color::Color};

mod _core {
    pub use aoide_core::{collection::*, entity::EntityHeader};
}

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaSourceConfig {
    pub source_path: SourcePathConfig,
}

impl TryFrom<MediaSourceConfig> for _core::MediaSourceConfig {
    type Error = anyhow::Error;

    fn try_from(from: MediaSourceConfig) -> anyhow::Result<Self> {
        let MediaSourceConfig { source_path } = from;
        let into = Self {
            source_path: source_path.try_into()?,
        };
        Ok(into)
    }
}

impl From<_core::MediaSourceConfig> for MediaSourceConfig {
    fn from(from: _core::MediaSourceConfig) -> Self {
        let _core::MediaSourceConfig { source_path } = from;
        Self {
            source_path: source_path.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Collection {
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    pub media_source_config: MediaSourceConfig,
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
        Self(from.hdr.into(), from.body.into())
    }
}
