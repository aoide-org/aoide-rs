// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// rustflags
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(clippy::pedantic)]
// Repetitions of module/type names occur frequently when using many
// modules for keeping the size of the source files handy. Often
// types have the same name as their parent module.
#![allow(clippy::module_name_repetitions)]
// Repeating the type name in `..Default::default()` expressions
// is not needed since the context is obvious.
#![allow(clippy::default_trait_access)]
// Using wildcard imports consciously is acceptable.
#![allow(clippy::wildcard_imports)]
// Importing all enum variants into a narrow, local scope is acceptable.
#![allow(clippy::enum_glob_use)]
// TODO: Review type casts
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
// TODO: Either globally allow exceptions from `clippy::pedantic`
// and comment why or fix the root causes in the code.
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
// rustdocflags
#![warn(rustdoc::broken_intra_doc_links)]
#![cfg_attr(not(test), deny(clippy::panic_in_result_fn))]
#![cfg_attr(not(debug_assertions), deny(clippy::used_underscore_binding))]

// TODO: Move into `domain` submodule
pub mod album;
pub mod audio;
pub mod collection;
pub mod entity;
pub mod media;
pub mod music;
pub mod playlist;
pub mod tag;
pub mod track;
pub mod util;

pub mod prelude {

    pub(crate) use crate::{
        entity::*,
        util::{clock::*, color::*, *},
    };

    pub(crate) use semval::prelude::*;
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
