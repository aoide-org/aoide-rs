// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use schemars::JsonSchema;

use aoide_core_json::entity::{EntityHeader, EntityRevision};

use crate::prelude::*;

mod _inner {
    pub use crate::_inner::track::find_unsynchronized::*;
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub struct UnsynchronizedMediaSource {
    pub path: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_rev: Option<u64>,
}

#[cfg(feature = "frontend")]
impl From<UnsynchronizedMediaSource> for _inner::UnsynchronizedMediaSource {
    fn from(from: UnsynchronizedMediaSource) -> Self {
        let UnsynchronizedMediaSource { path, external_rev } = from;
        Self {
            path: path.into(),
            external_rev,
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::UnsynchronizedMediaSource> for UnsynchronizedMediaSource {
    fn from(from: _inner::UnsynchronizedMediaSource) -> Self {
        let _inner::UnsynchronizedMediaSource { path, external_rev } = from;
        Self {
            path: path.into(),
            external_rev,
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub struct UnsynchronizedTrack {
    pub media_source: UnsynchronizedMediaSource,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_source_synchronized_rev: Option<EntityRevision>,
}

#[cfg(feature = "frontend")]
impl From<UnsynchronizedTrack> for _inner::UnsynchronizedTrack {
    fn from(from: UnsynchronizedTrack) -> Self {
        let UnsynchronizedTrack {
            media_source,
            media_source_synchronized_rev,
        } = from;
        Self {
            media_source: media_source.into(),
            media_source_synchronized_rev: media_source_synchronized_rev.map(Into::into),
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::UnsynchronizedTrack> for UnsynchronizedTrack {
    fn from(from: _inner::UnsynchronizedTrack) -> Self {
        let _inner::UnsynchronizedTrack {
            media_source,
            media_source_synchronized_rev,
        } = from;
        Self {
            media_source: media_source.into(),
            media_source_synchronized_rev: media_source_synchronized_rev.map(Into::into),
        }
    }
}

#[derive(Debug, JsonSchema)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
pub struct UnsynchronizedTrackEntity(EntityHeader, UnsynchronizedTrack);

#[cfg(feature = "frontend")]
impl From<UnsynchronizedTrackEntity> for _inner::UnsynchronizedTrackEntity {
    fn from(from: UnsynchronizedTrackEntity) -> Self {
        let UnsynchronizedTrackEntity(hdr, body) = from;
        Self::new(hdr, body)
    }
}

#[cfg(feature = "backend")]
impl From<_inner::UnsynchronizedTrackEntity> for UnsynchronizedTrackEntity {
    fn from(from: _inner::UnsynchronizedTrackEntity) -> Self {
        Self(from.hdr.into(), from.body.into())
    }
}
