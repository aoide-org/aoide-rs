// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

// Opt-in for allowed-by-default lints (in alphabetical order)
// See also: <https://doc.rust-lang.org/rustc/lints>
#![warn(future_incompatible)]
#![warn(let_underscore)]
#![warn(missing_debug_implementations)]
//#![warn(missing_docs)] // TODO
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(unreachable_pub)]
#![warn(unsafe_code)]
#![warn(unused)]
// Clippy lints
#![warn(clippy::pedantic)]
// Additional restrictions
#![warn(clippy::clone_on_ref_ptr)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::self_named_module_files)]

#[cfg(feature = "backend-embedded")]
pub use aoide_backend_embedded as backend_embedded;
pub use aoide_core::*;
#[cfg(feature = "api")]
pub use aoide_core_api as api;
#[cfg(all(feature = "api", feature = "json"))]
pub use aoide_core_api_json as api_json;
#[cfg(feature = "json")]
pub use aoide_core_json as json;
#[cfg(feature = "desktop-app")]
pub use aoide_desktop_app as desktop_app;
#[cfg(feature = "media-file")]
pub use aoide_media_file as media_file;
#[cfg(feature = "repo")]
pub use aoide_repo as repo;
#[cfg(all(feature = "repo", feature = "sqlite"))]
pub use aoide_repo_sqlite as repo_sqlite;
#[cfg(feature = "tantivy")]
pub use aoide_search_index_tantivy as search_index_tantivy;
#[cfg(feature = "sqlite")]
pub use aoide_storage_sqlite as storage_sqlite;
#[cfg(feature = "usecases")]
pub use aoide_usecases as usecases;
#[cfg(all(feature = "usecases", feature = "sqlite"))]
pub use aoide_usecases_sqlite as usecases_sqlite;
