// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub use aoide_core::*;

pub mod media {
    pub use aoide_core::media::*;

    #[cfg(feature = "media")]
    pub use aoide_media::*;
}

#[cfg(feature = "api")]
pub use aoide_core_api as api;

#[cfg(feature = "json")]
pub use aoide_core_json as json;

#[cfg(all(feature = "api", feature = "json"))]
pub use aoide_core_api_json as api_json;

#[cfg(feature = "backend-embedded")]
pub use aoide_backend_embedded as backend_embedded;

#[cfg(feature = "desktop-app")]
pub use aoide_desktop_app as desktop_app;

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
