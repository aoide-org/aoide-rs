// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(rustdoc::broken_intra_doc_links)]
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
// TODO: Add missing docs
#![allow(clippy::missing_errors_doc)]

pub mod action;
pub mod message;
pub mod state;
pub mod util;

pub mod models;

pub mod messaging;

#[cfg(feature = "webapi-backend")]
pub mod webapi;

pub mod prelude {
    pub use aoide_core::collection::EntityUid as CollectionUid;
}
