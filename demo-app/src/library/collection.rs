// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};

use discro::{Publisher, Subscriber};

use aoide::{
    backend_embedded::media::predefined_faceted_tag_mapping_config,
    desktop_app::{collection, Handle},
    media_file::io::import::ImportTrackConfig,
};

use super::{LibraryEventEmitter, LibraryNotification};

// Re-exports
pub use collection::*;

pub(super) async fn watch_state<E>(mut subscriber: Subscriber<State>, event_emitter: Weak<E>)
where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        let Some(event_emitter) = event_emitter.upgrade() else {
            log::info!("Stop watching collection state after event emitter has been dropped");
            break;
        };
        // The lock is released immediately after cloning the state.
        let state = subscriber.read_ack().clone();
        event_emitter.emit_notification(LibraryNotification::CollectionStateChanged(state.clone()));
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching collection state after publisher has been dropped");
            break;
        }
    }
}

pub struct RescanTask {
    started_at: Instant,
    progress:
        Subscriber<Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>>,
    join_handle: tokio::task::JoinHandle<
        anyhow::Result<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>,
    >,
}

impl RescanTask {
    pub fn spawn(
        rt: &tokio::runtime::Handle,
        handle: Handle,
        collection: Arc<collection::ObservableState>,
        track_search: Weak<super::track_search::ObservableState>,
    ) -> Self {
        let started_at = Instant::now();
        let progress_pub = Publisher::new(None);
        let progress = progress_pub.subscribe();
        let report_progress_fn = {
            // TODO: How to avoid wrapping the publisher?
            let progress_pub = Arc::new(Mutex::new(progress_pub));
            move |progress: Option<
                aoide::backend_embedded::batch::synchronize_collection_vfs::Progress,
            >| {
                progress_pub.lock().unwrap().write(progress);
            }
        };
        let task = synchronize_music_dir_task(handle, collection, report_progress_fn);
        let join_handle = rt.spawn(async move {
            let outcome = task.await?;
            // Discard any cached search results.
            let Some(track_search) = track_search.upgrade() else {
                return Ok(outcome);
            };
            track_search.reset_fetched();
            Ok(outcome)
        });
        Self {
            started_at,
            progress,
            join_handle,
        }
    }

    pub const fn started_at(&self) -> Instant {
        self.started_at
    }

    pub const fn progress(
        &self,
    ) -> &Subscriber<Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>>
    {
        &self.progress
    }

    pub fn abort(&self) {
        self.join_handle.abort();
    }

    pub fn is_finished(&self) -> bool {
        self.join_handle.is_finished()
    }

    pub async fn join(
        self,
    ) -> anyhow::Result<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome> {
        self.join_handle.await?
    }
}

fn synchronize_music_dir_task(
    handle: Handle,
    state: Arc<ObservableState>,
    mut report_progress_fn: impl FnMut(Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>)
        + Clone
        + Send
        + 'static,
) -> impl Future<
    Output = anyhow::Result<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>,
> + Send
       + 'static {
    let mut uid = None;
    state.modify(|state| {
        uid = state.entity_uid().map(ToOwned::to_owned);
        uid.is_some() && state.reset_to_pending()
    });
    async move {
        let Some(uid) = uid else {
            anyhow::bail!("No collection");
        };
        log::debug!("Synchronizing collection with music directory...");
        let import_track_config = ImportTrackConfig {
            // TODO: Customize faceted tag mapping
            faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
            ..Default::default()
        };
        let res = {
            let mut report_progress_fn = report_progress_fn.clone();
            let report_progress_fn = move |progress| {
                report_progress_fn(Some(progress));
            };
            synchronize_vfs(&handle, uid, import_track_config, report_progress_fn).await
        };
        report_progress_fn(None);
        log::debug!(
            "Synchronizing collection with music directory finished: {:?}",
            res
        );
        if let Err(err) = state.refresh_from_db(&handle).await {
            log::warn!("Failed to refresh collection: {err}");
        }
        res
    }
}
