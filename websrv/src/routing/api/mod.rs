// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ops::Deref,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use axum::{Router, extract::FromRef};

mod collection;
mod media_source;
mod media_tracker;
mod playlist;
#[cfg(feature = "json-schema")]
mod schema;
mod storage;
mod track;

/// Flag for aborting the currently running, long-running task.
#[derive(Debug, Clone, Default)]
struct AbortFlag(Arc<AtomicBool>);

impl AbortFlag {
    fn reset(&self) {
        self.store(false, Ordering::Relaxed);
    }

    fn abort(&self) {
        self.store(true, Ordering::Relaxed);
    }
}

impl Deref for AbortFlag {
    type Target = AtomicBool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone, FromRef)]
struct ApiState {
    gatekeeper: Arc<Gatekeeper>,
    abort_flag: AbortFlag,
    media_tracker_progress: media_tracker::SharedProgress,
}

pub(crate) fn create_router(gatekeeper: Arc<Gatekeeper>) -> Router {
    log::info!("Creating API routes");

    let state = ApiState {
        gatekeeper,
        abort_flag: Default::default(),
        media_tracker_progress: Default::default(),
    };

    let router = Router::new()
        .nest("/c", collection::router())
        .nest("/c/{collection_uid}/mt", media_tracker::collected_router())
        .nest("/c/{collection_uid}/ms", media_source::collected_router())
        .nest("/c/{collection_uid}/t", track::collected_router())
        .nest("/c/{collection_uid}/p", playlist::collected_router())
        .nest("/mt", media_tracker::router())
        .nest("/t", track::router())
        .nest("/p", playlist::router())
        .nest("/storage", storage::router());

    #[cfg(feature = "json-schema")]
    let router = router.nest("/schema", schema::router());

    router.with_state(state)
}
