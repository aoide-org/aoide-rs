// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    fmt,
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, Mutex, Weak},
    time::Instant,
};

use aoide::{
    api::media::source::ResolveUrlFromContentPath,
    desktop_app::{settings, track, Handle},
};
use discro::{Publisher, Subscriber};

mod collection;
mod track_search;

const CREATE_NEW_COLLECTION_ENTITY_IF_NOT_FOUND: bool = true;

const NESTED_MUSIC_DIRS: collection::NestedMusicDirectoriesStrategy =
    collection::NestedMusicDirectoriesStrategy::Permit;

// We always need the URL in addition to the virtual file path
const RESOLVE_TRACK_URL_FROM_CONTENT_PATH: Option<ResolveUrlFromContentPath> =
    Some(ResolveUrlFromContentPath::CanonicalRootUrl);

fn default_track_search_params() -> aoide::api::track::search::Params {
    aoide::api::track::search::Params {
        resolve_url_from_content_path: RESOLVE_TRACK_URL_FROM_CONTENT_PATH.clone(),
        ..Default::default()
    }
}

const TRACK_REPO_SEARCH_PREFETCH_LIMIT_USIZE: usize = 100;
const TRACK_REPO_SEARCH_PREFETCH_LIMIT: NonZeroUsize =
    NonZeroUsize::MIN.saturating_add(TRACK_REPO_SEARCH_PREFETCH_LIMIT_USIZE - 1);

pub type TrackSearchStateRef<'r> = discro::Ref<'r, track_search::State>;

#[derive(Clone)]
pub struct TrackSearchStateReader {
    subscriber: discro::Subscriber<track::repo_search::State>,
}

impl TrackSearchStateReader {
    /// Read the current state
    ///
    /// Holds a read lock until the returned reference is dropped.
    pub fn read(&self) -> TrackSearchStateRef<'_> {
        self.subscriber.read()
    }
}

impl fmt::Debug for TrackSearchStateReader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TrackSearchStateReader").finish()
    }
}

#[derive(Debug)]
#[allow(clippy::enum_variant_names)] // common `...Changed` suffix
pub enum LibraryNotification {
    SettingsStateChanged(settings::State),
    CollectionStateChanged(collection::State),
    TrackSearchStateChanged(TrackSearchStateReader),
}

/// Library event emitter.
///
/// No locks must be held when calling `emit_notification()`!
pub trait LibraryEventEmitter: Send + Sync + 'static {
    fn emit_notification(&self, notification: LibraryNotification);
}

/// Stateful library frontend.
///
/// Manages the application state that should not depend on any
/// particular UI technology.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct LibraryState {
    settings: Arc<settings::ObservableState>,
    collection: Arc<collection::ObservableState>,
    track_search: Arc<track::repo_search::ObservableState>,
}

impl LibraryState {
    #[must_use]
    pub fn new(initial_settings: settings::State) -> Self {
        let initial_track_search = track::repo_search::State::new(default_track_search_params());
        Self {
            settings: Arc::new(settings::ObservableState::new(initial_settings)),
            collection: Arc::default(),
            track_search: Arc::new(track::repo_search::ObservableState::new(
                initial_track_search,
            )),
        }
    }

    /// Observable settings state.
    #[must_use]
    pub const fn settings(&self) -> &Arc<settings::ObservableState> {
        &self.settings
    }

    /// Observable collection state.
    #[must_use]
    pub const fn collection(&self) -> &Arc<collection::ObservableState> {
        &self.collection
    }

    /// Observable track (repo) search state.
    #[must_use]
    pub const fn track_search(&self) -> &Arc<track::repo_search::ObservableState> {
        &self.track_search
    }
}

/// Library state with a handle to the runtime environment
#[allow(missing_debug_implementations)]
pub struct Library {
    handle: Handle,
    state: LibraryState,
    pending_rescan_collection_task: Option<RescanCollectionTask>,
}

impl Library {
    #[must_use]
    pub fn new(handle: Handle, initial_settings: settings::State) -> Self {
        Self {
            handle,
            state: LibraryState::new(initial_settings),
            pending_rescan_collection_task: None,
        }
    }

    #[must_use]
    pub const fn handle(&self) -> &Handle {
        &self.handle
    }

    #[must_use]
    pub const fn state(&self) -> &LibraryState {
        &self.state
    }

    pub fn update_music_directory(&self, music_dir: Option<PathBuf>) {
        let music_dir = music_dir.map(Into::into);
        self.state.settings().modify(|state| {
            if music_dir == state.music_dir {
                log::debug!("Music directory unchanged: {music_dir:?}");
                return false;
            }
            let old_music_dir = state.music_dir.take();
            log::debug!("Updating music directory: {old_music_dir:?} -> {music_dir:?}");
            state.music_dir = music_dir;
            true
        });
    }

    pub fn reset_music_directory(&self) {
        self.update_music_directory(None);
    }

    pub fn reset_collection(&self) {
        self.state.collection().modify(collection::State::reset);
    }

    pub fn spawn_rescan_collection_task(&mut self, rt: &tokio::runtime::Handle) -> bool {
        if let Some(rescan_collection_task) = self.pending_rescan_collection_task.as_ref() {
            if rescan_collection_task.join_handle.is_finished() {
                log::info!("Resetting finished rescan collection task");
                self.pending_rescan_collection_task = None;
            } else {
                log::info!("Rescan collection still pending");
                return false;
            }
        }
        log::info!("Spawning rescan collection task");
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
        let handle = self.handle.clone();
        let collection_state = Arc::clone(self.state.collection());
        let task =
            collection::synchronize_music_dir_task(handle, collection_state, report_progress_fn);
        let track_search_state = Arc::downgrade(self.state.track_search());
        let join_handle = rt.spawn(async move {
            let outcome = task.await?;
            // Discard any cached search results.
            let Some(track_search_state) = track_search_state.upgrade() else {
                return Ok(outcome);
            };
            track_search_state.reset_fetched();
            Ok(outcome)
        });
        self.pending_rescan_collection_task = Some(RescanCollectionTask {
            started_at,
            progress,
            join_handle,
        });
        true
    }

    pub const fn pending_rescan_collection_task(&self) -> Option<&RescanCollectionTask> {
        self.pending_rescan_collection_task.as_ref()
    }

    pub fn abort_pending_rescan_collection_task(&mut self) -> Option<RescanCollectionTask> {
        let pending_rescan_collection_task = self.pending_rescan_collection_task.take();
        let Some(rescan_collection_task) = pending_rescan_collection_task else {
            return None;
        };
        log::info!("Aborting rescan collection task");
        rescan_collection_task.abort();
        Some(rescan_collection_task)
    }

    pub fn search_tracks(&self, input: &str) {
        let filter = track_search::parse_filter_from_input(input);
        let resolve_url_from_content_path = self
            .state
            .track_search()
            .read()
            .default_params()
            .resolve_url_from_content_path
            .clone();
        let mut params = aoide::api::track::search::Params {
            filter,
            ordering: vec![], // TODO
            resolve_url_from_content_path,
        };
        // Argument is consumed when updating succeeds
        if self.state.track_search().update_params(&mut params) {
            log::debug!("Track search params updated: {params:?}");
        } else {
            log::debug!("Track search params not updated: {params:?}");
        }
    }

    /// Spawn reactive background tasks
    pub fn spawn_background_tasks(&self, tokio_rt: &tokio::runtime::Handle, settings_dir: PathBuf) {
        tokio_rt.spawn(settings::tasklet::on_state_changed_save_to_file(
            self.state.settings.subscribe_changed(),
            settings_dir,
            |err| {
                log::error!("Failed to save settings to file: {err}");
            },
        ));
        tokio_rt.spawn(collection::tasklet::on_settings_changed(
            Arc::downgrade(&self.state.settings),
            Arc::downgrade(&self.state.collection),
            Handle::downgrade(&self.handle),
            CREATE_NEW_COLLECTION_ENTITY_IF_NOT_FOUND,
            NESTED_MUSIC_DIRS,
            |err| {
                log::error!("Failed to update collection after settings state changed: {err}");
            },
        ));
        tokio_rt.spawn(track::repo_search::tasklet::on_collection_changed(
            Arc::downgrade(&self.state.collection),
            Arc::downgrade(&self.state.track_search),
        ));
        tokio_rt.spawn(track::repo_search::tasklet::on_should_prefetch(
            Arc::downgrade(&self.state.track_search),
            Handle::downgrade(&self.handle),
            Some(TRACK_REPO_SEARCH_PREFETCH_LIMIT),
        ));
    }

    pub fn spawn_notification_tasks<E>(
        &self,
        tokio_rt: &tokio::runtime::Handle,
        event_emitter: &Arc<E>,
    ) where
        E: LibraryEventEmitter,
    {
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let subscriber = self.state().settings().subscribe_changed();
            async move {
                watch_settings_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let subscriber = self.state().collection().subscribe_changed();
            async move {
                watch_collection_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let subscriber = self.state().track_search().subscribe_changed();
            async move {
                watch_track_search_state(subscriber, event_emitter).await;
            }
        });
    }
}

async fn watch_settings_state<E>(
    mut subscriber: Subscriber<settings::State>,
    event_emitter: Weak<E>,
) where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        let Some(event_emitter) = event_emitter.upgrade() else {
            log::info!("Stop watching settings state after event emitter has been dropped");
            break;
        };
        // The lock is released immediately after cloning the state.
        let state = subscriber.read_ack().clone();
        event_emitter.emit_notification(LibraryNotification::SettingsStateChanged(state.clone()));
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching settings state after publisher has been dropped");
            break;
        }
    }
}

async fn watch_collection_state<E>(
    mut subscriber: Subscriber<collection::State>,
    event_emitter: Weak<E>,
) where
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

async fn watch_track_search_state<E>(
    mut subscriber: Subscriber<track::repo_search::State>,
    event_emitter: Weak<E>,
) where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        let Some(event_emitter) = event_emitter.upgrade() else {
            log::info!("Stop watching track search state after event emitter has been dropped");
            break;
        };
        let reader = TrackSearchStateReader {
            subscriber: subscriber.clone(),
        };
        event_emitter.emit_notification(LibraryNotification::TrackSearchStateChanged(reader));
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching track search state after publisher has been dropped");
            break;
        }
    }
}

pub struct RescanCollectionTask {
    started_at: Instant,
    progress:
        Subscriber<Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>>,
    join_handle: tokio::task::JoinHandle<
        anyhow::Result<aoide::backend_embedded::batch::synchronize_collection_vfs::Outcome>,
    >,
}

impl RescanCollectionTask {
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
