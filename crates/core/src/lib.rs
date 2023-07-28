// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::self_named_module_files)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `Default::default()` expressions is not needed
// as long as the context is obvious.
#![allow(clippy::default_trait_access)]
// Using wildcard imports consciously is acceptable.
#![allow(clippy::wildcard_imports)]
// Importing all enum variants into a narrow, local scope is acceptable.
#![allow(clippy::enum_glob_use)]
// TODO: Review type casts
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

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

mod compat {
    use std::cmp::Ordering;

    // TODO: Remove after https://github.com/rust-lang/rust/issues/53485
    // has been stabilized.
    pub(crate) fn is_sorted_by<'a, T, F>(
        iterable: impl IntoIterator<Item = &'a T>,
        mut cmp: F,
    ) -> bool
    where
        F: FnMut(&'a T, &'a T) -> Ordering,
        T: 'a,
    {
        let mut iter = iterable.into_iter();
        if let Some(first) = iter.next() {
            let mut prev = first;
            for next in iter {
                if cmp(prev, next) == Ordering::Greater {
                    return false;
                }
                prev = next;
            }
        }
        true
    }
}
