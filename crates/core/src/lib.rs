// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod audio;
pub mod media;
pub mod music;
pub mod tag;
pub mod util;

mod album;
pub use self::album::AlbumSummary;

mod entity;
pub use self::entity::*;

pub mod collection;
pub use self::collection::{
    Collection, Entity as CollectionEntity, EntityHeader as CollectionHeader,
    EntityUid as CollectionUid,
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

pub mod prelude {
    // Re-export main type and trait methods from nonicle
    pub use nonicle::{Canonical, Canonicalize as _, CanonicalizeInto as _, IsCanonical as _};
    pub(crate) use semval::prelude::*;
    // Re-export trait methods from semval
    pub use semval::{IntoValidated as _, IsValid, Validate as _, ValidatedFrom as _};

    pub(crate) use crate::{
        entity::*,
        util::{clock::*, color::*, *},
    };
}
