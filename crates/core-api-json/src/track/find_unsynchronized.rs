// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_json::{
    entity::{EntityHeader, EntityRevision},
    media::content::ContentLink,
};

use crate::prelude::*;

mod _inner {
    pub(super) use crate::_inner::track::find_unsynchronized::*;
}

#[derive(Debug)]
#[cfg_attr(feature = "frontend", derive(Deserialize))]
#[cfg_attr(feature = "backend", derive(Serialize))]
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
#[cfg_attr(feature = "schemars", derive(JsonSchema))]
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
        let (hdr, body) = from.into();
        Self(hdr.into(), body.into())
    }
}
