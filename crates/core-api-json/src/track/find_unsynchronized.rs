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

use aoide_core_json::{
    entity::{EntityHeader, EntityRevision},
    media::content::ContentLink,
};

use crate::prelude::*;

mod _inner {
    pub use crate::_inner::track::find_unsynchronized::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct UnsynchronizedTrack {
    pub content_link: ContentLink,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synchronized_rev: Option<EntityRevision>,
}

#[cfg(feature = "frontend")]
impl From<UnsynchronizedTrack> for _inner::UnsynchronizedTrack {
    fn from(from: UnsynchronizedTrack) -> Self {
        let UnsynchronizedTrack {
            content_link,
            last_synchronized_rev,
        } = from;
        Self {
            content_link: content_link.into(),
            last_synchronized_rev: last_synchronized_rev.map(Into::into),
        }
    }
}

#[cfg(feature = "backend")]
impl From<_inner::UnsynchronizedTrack> for UnsynchronizedTrack {
    fn from(from: _inner::UnsynchronizedTrack) -> Self {
        let _inner::UnsynchronizedTrack {
            content_link,
            last_synchronized_rev,
        } = from;
        Self {
            content_link: content_link.into(),
            last_synchronized_rev: last_synchronized_rev.map(Into::into),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "with-schemars", derive(JsonSchema))]
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
