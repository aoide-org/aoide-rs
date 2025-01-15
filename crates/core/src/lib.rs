// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod audio;
pub mod music;
pub mod util;

mod album;
pub use self::album::AlbumSummary;

pub mod collection;
pub use self::collection::{
    Collection, Entity as CollectionEntity, EntityHeader as CollectionHeader,
    EntityUid as CollectionUid,
};

mod entity;
pub use self::entity::*;

pub mod media;
pub use self::media::Source as MediaSource;

pub mod tag;
pub use self::tag::{
    FacetId as TagFacetId, FacetedTags, Label as TagLabel, PlainTag, Score as TagScore, TagsMap,
};

pub mod track;
pub use self::track::{
    Entity as TrackEntity, EntityBody as TrackBody, EntityHeader as TrackHeader,
    EntityUid as TrackUid, Track,
};

pub mod playlist;
pub use self::playlist::{
    Entity as PlaylistEntity, EntityHeader as PlaylistHeader, EntityUid as PlaylistUid, Playlist,
};
