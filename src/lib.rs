// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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

pub mod prelude {
    // Avoid transitive dependencies on nonicle.
    pub use nonicle::{
        CanonicalOrd as _, Canonicalize as _, CanonicalizeInto as _, IsCanonical as _,
    };

    // Avoid transitive dependencies on semval.
    pub use semval::prelude::*;
}
