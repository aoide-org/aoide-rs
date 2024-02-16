// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

use aoide::{
    api::media::source::ResolveUrlFromContentPath,
    desktop_app::{
        collection::SynchronizeVfsTask, fs::DirPath, track::repo_search::FetchMoreSucceeded,
        Handle, ObservableReader,
    },
};
use discro::Ref;

use crate::NoReceiverForEvent;

pub mod collection;
pub mod settings;
pub mod track_search;

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

#[derive(Debug)]
#[allow(clippy::enum_variant_names)] // common `...Changed` suffix
pub enum LibraryEvent {
    SettingsStateChanged,
    CollectionStateChanged,
    TrackSearchStateChanged,
    FetchMoreTrackSearchResultsFinished(anyhow::Result<FetchMoreSucceeded>),
}

/// Library event emitter.
///
/// No locks must be held when calling `emit_event()`!
pub trait LibraryEventEmitter: Send + Sync + 'static {
    fn emit_event(&self, event: LibraryEvent) -> Result<(), NoReceiverForEvent>;
}

/// Stateful library frontend.
///
/// Manages the application state that should not depend on any
/// particular UI technology.
#[allow(missing_debug_implementations)]
pub struct LibraryState {
    settings: Arc<settings::ObservableState>,
    collection: Arc<collection::ObservableState>,
    track_search: Arc<track_search::ObservableState>,
    pending_synchronize_music_dir_task: Option<SynchronizeVfsTask>,
}

impl LibraryState {
    #[must_use]
    pub fn new(initial_settings: settings::State) -> Self {
        let initial_track_search = track_search::State::new(default_track_search_params());
        Self {
            settings: Arc::new(settings::ObservableState::new(initial_settings)),
            collection: Arc::default(),
            track_search: Arc::new(track_search::ObservableState::new(initial_track_search)),
            pending_synchronize_music_dir_task: None,
        }
    }

    /// Readable settings state.
    #[must_use]
    pub fn settings(&self) -> &impl ObservableReader<settings::State> {
        self.settings.as_ref()
    }

    /// Observable collection state.
    #[must_use]
    pub fn collection(&self) -> &impl ObservableReader<collection::State> {
        self.collection.as_ref()
    }

    /// Observable track (repo) search state.
    #[must_use]
    pub fn track_search(&self) -> &impl ObservableReader<track_search::State> {
        self.track_search.as_ref()
    }

    /// Read-lock the current state.
    #[must_use]
    pub fn read_current(&self) -> CurrentLibraryState<'_> {
        let Self {
            settings,
            collection,
            track_search,
            pending_synchronize_music_dir_task,
        } = self;
        CurrentLibraryState {
            settings: settings.read(),
            collection: collection.read(),
            track_search: track_search.read(),
            pending_synchronize_music_dir_task: pending_synchronize_music_dir_task.as_ref(),
        }
    }
}

pub struct CurrentLibraryState<'a> {
    settings: Ref<'a, settings::State>,
    collection: Ref<'a, collection::State>,
    track_search: Ref<'a, track_search::State>,
    pending_synchronize_music_dir_task: Option<&'a SynchronizeVfsTask>,
}

impl CurrentLibraryState<'_> {
    #[must_use]
    pub fn settings(&self) -> &settings::State {
        &self.settings
    }

    #[must_use]
    pub fn collection(&self) -> &collection::State {
        &self.collection
    }

    #[must_use]
    pub fn track_search(&self) -> &track_search::State {
        &self.track_search
    }

    pub fn could_reset_music_dir(&self) -> bool {
        self.settings().music_dir.is_some()
    }

    pub fn could_spawn_synchronize_music_dir_task(&self) -> bool {
        if !self.collection().is_ready() {
            return false;
        }
        let Some(pending_task) = self.pending_synchronize_music_dir_task else {
            return true;
        };
        pending_task.is_finished()
    }

    pub fn could_abort_synchronize_music_dir_task(&self) -> bool {
        let Some(pending_task) = self.pending_synchronize_music_dir_task else {
            return false;
        };
        !pending_task.is_finished()
    }

    pub fn could_search_tracks(&self) -> bool {
        !self.collection().is_ready() && self.track_search().pending_since().is_none()
    }

    pub fn could_spawn_fetch_more_track_search_results(&self) -> bool {
        self.track_search().can_fetch_more().unwrap_or(false)
    }
}

/// Library state with a handle to the runtime environment
#[allow(missing_debug_implementations)]
pub struct Library {
    handle: Handle,
    state: LibraryState,
}

impl Library {
    #[must_use]
    pub fn new(handle: Handle, initial_settings: settings::State) -> Self {
        Self {
            handle,
            state: LibraryState::new(initial_settings),
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

    pub fn update_music_dir(&self, music_dir: Option<&DirPath<'_>>) {
        if self.state.settings.update_music_dir(music_dir) {
            log::info!("Music directory updated: {music_dir:?}");
        } else {
            log::debug!("Music directory unchanged: {music_dir:?}");
        }
    }

    pub fn reset_music_dir(&self) {
        self.update_music_dir(None);
    }

    pub fn spawn_synchronize_music_dir_task(&mut self, rt: &tokio::runtime::Handle) -> bool {
        if let Some(synchronize_music_dir_task) =
            self.state.pending_synchronize_music_dir_task.as_ref()
        {
            if synchronize_music_dir_task.is_finished() {
                log::info!("Resetting synchronize music directory task after finished");
                self.state.pending_synchronize_music_dir_task = None;
            } else {
                log::info!("Synchronize music directory task still pending");
                return false;
            }
        }
        log::info!("Spawning synchronize music directory task");
        let handle = self.handle.clone();
        let collection = Arc::clone(&self.state.collection);
        let synchronize_music_dir_task = SynchronizeVfsTask::spawn(rt, handle, collection);
        self.state.pending_synchronize_music_dir_task = Some(synchronize_music_dir_task);
        true
    }

    pub fn on_collection_state_changed(&mut self, collection_state: &collection::State) -> bool {
        let mut changed = false;
        if self.state.pending_synchronize_music_dir_task.is_some()
            && !matches!(collection_state, collection::State::Synchronizing { .. })
        {
            // The task will eventually finish.
            log::debug!("Resetting pending synchronize music directory task");
            self.state.pending_synchronize_music_dir_task = None;
            changed = true;
        }
        changed
    }

    pub fn abort_pending_synchronize_music_dir_task(&mut self) -> Option<SynchronizeVfsTask> {
        let pending_synchronize_music_dir_task =
            self.state.pending_synchronize_music_dir_task.take();
        let Some(synchronize_music_dir_task) = pending_synchronize_music_dir_task else {
            return None;
        };
        log::info!("Aborting synchronize music directory task");
        synchronize_music_dir_task.abort();
        Some(synchronize_music_dir_task)
    }

    pub fn search_tracks(&self, input: &str) {
        let filter = track_search::parse_filter_from_input(input);
        let resolve_url_from_content_path = self
            .state
            .track_search()
            .read_observable()
            .default_params()
            .resolve_url_from_content_path
            .clone();
        let mut params = aoide::api::track::search::Params {
            filter,
            ordering: vec![], // TODO
            resolve_url_from_content_path,
        };
        // Argument is consumed when updating succeeds
        log::debug!("Updating track search params: {params:?}");
        if !self.state.track_search.update_params(&mut params) {
            log::debug!("Track search params not updated: {params:?}");
        }
    }

    pub fn spawn_fetch_more_track_search_results<E>(
        &self,
        tokio_rt: &tokio::runtime::Handle,
        event_emitter: &E,
    ) where
        E: LibraryEventEmitter + Clone + 'static,
    {
        let Some(fetch_more_task) = self
            .state
            .track_search
            .fetch_more_task(&self.handle, Some(TRACK_REPO_SEARCH_PREFETCH_LIMIT))
        else {
            return;
        };
        let event_emitter = event_emitter.clone();
        tokio_rt.spawn(async move {
            log::debug!("Fetching more track search results");
            let result = fetch_more_task.await;
            if let Err(err) =
                event_emitter.emit_event(LibraryEvent::FetchMoreTrackSearchResultsFinished(result))
            {
                log::warn!("Failed to emit event after Fetching more track search results finished: {err:?}");
            }
        });
    }

    pub fn fetch_more_track_search_results_finished(
        &self,
        result: anyhow::Result<FetchMoreSucceeded>,
    ) {
        self.state.track_search.fetch_more_task_finished(result);
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
        tokio_rt.spawn(collection::tasklet::on_settings_state_changed(
            &self.state.settings,
            Arc::downgrade(&self.state.collection),
            Handle::downgrade(&self.handle),
            CREATE_NEW_COLLECTION_ENTITY_IF_NOT_FOUND,
            NESTED_MUSIC_DIRS,
        ));
        tokio_rt.spawn(track_search::tasklet::on_collection_state_changed(
            &self.state.collection,
            Arc::downgrade(&self.state.track_search),
        ));
        tokio_rt.spawn(track_search::tasklet::on_should_prefetch(
            &self.state.track_search,
            Handle::downgrade(&self.handle),
            Some(TRACK_REPO_SEARCH_PREFETCH_LIMIT),
        ));
    }

    pub fn spawn_event_tasks<E>(&self, tokio_rt: &tokio::runtime::Handle, event_emitter: &E)
    where
        E: LibraryEventEmitter + Clone + 'static,
    {
        tokio_rt.spawn({
            let subscriber = self.state().settings.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                settings::watch_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let subscriber = self.state().collection.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                collection::watch_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let subscriber = self.state().track_search.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                track_search::watch_state(subscriber, event_emitter).await;
            }
        });
    }
}
