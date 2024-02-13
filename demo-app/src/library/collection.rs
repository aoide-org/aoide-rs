// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Instant,
};

use discro::{Publisher, Subscriber};

use aoide::{
    backend_embedded::media::predefined_faceted_tag_mapping_config,
    desktop_app::{collection, Handle},
    media_file::io::import::ImportTrackConfig,
};

use crate::NoReceiverForEvent;

use super::{LibraryEvent, LibraryEventEmitter};

// Re-exports
pub use collection::*;

pub type StateSubscriber = Subscriber<State>;

pub(super) async fn watch_state<E>(mut subscriber: StateSubscriber, event_emitter: E)
where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        drop(subscriber.read_ack());
        if let Err(NoReceiverForEvent) =
            event_emitter.emit_event(LibraryEvent::CollectionStateChanged)
        {
            log::info!("Stop watching collection state after event receiver has been dropped");
            break;
        };
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
        Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>,
    >,
}

impl RescanTask {
    pub fn spawn(
        rt: &tokio::runtime::Handle,
        handle: Handle,
        collection: Arc<collection::ObservableState>,
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
        let join_handle = rt.spawn(task);
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
    ) -> anyhow::Result<Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>>
    {
        self.join_handle.await.map_err(Into::into)
    }
}

#[allow(clippy::manual_async_fn)] // Required to specify the trait bounds of the returned `Future` explicitly.
fn synchronize_music_dir_task(
    handle: Handle,
    state: Arc<ObservableState>,
    mut report_progress_fn: impl FnMut(Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>)
        + Clone
        + Send
        + 'static,
) -> impl Future<Output = Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>>
       + Send
       + 'static {
    async move {
        log::debug!("Synchronizing collection with music directory...");
        let import_track_config = ImportTrackConfig {
            // TODO: Customize faceted tag mapping
            faceted_tag_mapping: predefined_faceted_tag_mapping_config(),
            ..Default::default()
        };
        let outcome = {
            let mut report_progress_fn = report_progress_fn.clone();
            let report_progress_fn = move |progress| {
                report_progress_fn(Some(progress));
            };
            state
                .synchronize_vfs(&handle, import_track_config, report_progress_fn)
                .await
        };
        report_progress_fn(None);
        log::debug!("Synchronizing collection with music directory finished: {outcome:?}");
        // Implicitly refresh the state from the database to reflect the changes.
        state.refresh_from_db(&handle).await;
        outcome
    }
}
