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

use crate::prelude::*;

use crate::{media::SourcePathKind, util::color::Color};

mod _core {
    pub use aoide_core::{collection::*, entity::EntityHeader};
}

use url::Url;

///////////////////////////////////////////////////////////////////////
// Collection
///////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MediaSourceConfig {
    pub path_kind: SourcePathKind,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_url: Option<Url>,
}

impl From<MediaSourceConfig> for _core::MediaSourceConfig {
    fn from(from: MediaSourceConfig) -> Self {
        let MediaSourceConfig {
            path_kind,
            root_url,
        } = from;
        Self {
            path_kind: path_kind.into(),
            root_url: root_url.map(Into::into),
        }
    }
}

impl From<_core::MediaSourceConfig> for MediaSourceConfig {
    fn from(from: _core::MediaSourceConfig) -> Self {
        let _core::MediaSourceConfig {
            path_kind,
            root_url,
        } = from;
        Self {
            path_kind: path_kind.into(),
            root_url: root_url.map(Into::into),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(test, derive(Eq, PartialEq))]
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

impl From<Collection> for _core::Collection {
    fn from(from: Collection) -> Self {
        let Collection {
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

impl From<Entity> for _core::Entity {
    fn from(from: Entity) -> Self {
        Self::new(from.0, from.1)
    }
}

impl From<_core::Entity> for Entity {
    fn from(from: _core::Entity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}
