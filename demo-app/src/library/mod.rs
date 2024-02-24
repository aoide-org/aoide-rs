// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, sync::Arc, time::Instant};

use aoide::{
    desktop_app::{collection::SynchronizeVfsTask, fs::DirPath, Handle, ObservableReader},
    media::content::ContentPath,
    CollectionUid,
};
use discro::Ref;

use crate::NoReceiverForEvent;

pub mod collection;
pub mod settings;
pub mod track_search;

#[derive(Debug)]
pub enum Event {
    Settings(settings::Event),
    Collection(collection::Event),
    TrackSearch(track_search::Event),
    MusicDirSyncProgress(
        Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>,
    ),
    MusicDirListResult {
        collection_uid: CollectionUid,
        params: aoide::api::media::tracker::count_sources_in_directories::Params,
        result: anyhow::Result<Vec<(ContentPath<'static>, usize)>>,
    },
}

impl From<settings::Event> for Event {
    fn from(event: settings::Event) -> Self {
        Self::Settings(event)
    }
}

impl From<collection::Event> for Event {
    fn from(event: collection::Event) -> Self {
        Self::Collection(event)
    }
}

impl From<track_search::Event> for Event {
    fn from(event: track_search::Event) -> Self {
        Self::TrackSearch(event)
    }
}

/// Event emitter.
///
/// No locks must be held when calling `emit_event()`!
pub trait EventEmitter: Send + Sync + 'static {
    fn emit_event(&self, event: Event) -> Result<(), NoReceiverForEvent>;
}

/// Stateful library frontend.
///
/// Manages the application state that should not depend on any
/// particular UI technology.
#[allow(missing_debug_implementations)]
pub struct StateObservables {
    pub settings: Arc<settings::ObservableState>,
    pub collection: Arc<collection::ObservableState>,
    pub track_search: Arc<track_search::ObservableState>,
}

impl StateObservables {
    #[must_use]
    fn new(initial_settings: settings::State) -> Self {
        let settings = Arc::new(settings::ObservableState::new(initial_settings));
        let collection = Arc::new(collection::ObservableState::default());
        let track_search = Arc::new(track_search::ObservableState::new(
            track_search::State::new(track_search::default_params()),
        ));
        Self {
            settings,
            collection,
            track_search,
        }
    }
}

/// Stateful library frontend.
///
/// Manages the application state that should not depend on any
/// particular UI technology.
#[derive(Debug, Default)]
pub struct State {
    pub music_dir: Option<DirPath<'static>>,
    pub collection: collection::State,
    pub track_search_memo: TrackSearchMemoState,
    pub pending_music_dir_sync_task: Option<SynchronizeVfsTask>,
}

#[derive(Debug)]
pub enum TrackSearchMemoState {
    Ready(track_search::Memo),
    Pending {
        memo: track_search::Memo,
        memo_delta: track_search::MemoDelta,
        state_changed_again: bool,
        pending_since: Instant,
    },
}

impl TrackSearchMemoState {
    #[must_use]
    pub fn new_pending(memo: track_search::Memo, memo_delta: track_search::MemoDelta) -> Self {
        Self::Pending {
            memo,
            memo_delta,
            state_changed_again: false,
            pending_since: Instant::now(),
        }
    }

    #[must_use]
    pub const fn pending_since(&self) -> Option<Instant> {
        match self {
            Self::Ready(_) => None,
            Self::Pending { pending_since, .. } => Some(*pending_since),
        }
    }
}

impl Default for TrackSearchMemoState {
    fn default() -> Self {
        Self::Ready(Default::default())
    }
}

impl State {
    #[must_use]
    fn read_lock_current<'a>(&'a self, observables: &'a StateObservables) -> CurrentState<'a> {
        let Self {
            music_dir,
            collection,
            track_search_memo,
            pending_music_dir_sync_task,
        } = self;
        let StateObservables { track_search, .. } = observables;
        let music_dir = music_dir.as_ref();
        let pending_music_dir_sync_task = pending_music_dir_sync_task.as_ref();
        let track_search = track_search.read_lock();
        CurrentState {
            music_dir,
            collection,
            track_search_memo,
            pending_music_dir_sync_task,
            track_search,
        }
    }
}

#[allow(missing_debug_implementations)]
pub struct CurrentState<'a> {
    music_dir: Option<&'a DirPath<'static>>,
    collection: &'a collection::State,
    track_search_memo: &'a TrackSearchMemoState,
    pending_music_dir_sync_task: Option<&'a SynchronizeVfsTask>,
    track_search: Ref<'a, track_search::State>,
}

impl CurrentState<'_> {
    #[must_use]
    pub const fn music_dir(&self) -> Option<&'_ DirPath<'static>> {
        self.music_dir
    }

    #[must_use]
    pub const fn collection(&self) -> &collection::State {
        self.collection
    }

    #[must_use]
    pub const fn track_search_memo(&self) -> &TrackSearchMemoState {
        self.track_search_memo
    }

    #[must_use]
    pub fn track_search(&self) -> &track_search::State {
        &self.track_search
    }

    #[must_use]
    pub const fn could_reset_music_dir(&self) -> bool {
        self.music_dir.is_some()
    }

    #[must_use]
    pub fn could_synchronize_music_dir_task(&self) -> bool {
        if !self.collection().is_ready() {
            return false;
        }
        let Some(pending_task) = self.pending_music_dir_sync_task else {
            return true;
        };
        pending_task.is_finished()
    }

    #[must_use]
    pub fn could_abort_synchronize_music_dir_task(&self) -> bool {
        let Some(pending_task) = self.pending_music_dir_sync_task else {
            return false;
        };
        !pending_task.is_finished()
    }

    #[must_use]
    pub fn could_view_music_dir_list(&self) -> bool {
        self.collection().is_ready() && !self.could_abort_synchronize_music_dir_task()
    }

    #[must_use]
    pub fn could_search_tracks(&self) -> bool {
        self.collection().is_ready() && self.track_search().pending_since().is_none()
    }

    #[must_use]
    pub fn could_fetch_more_track_search_results(&self) -> bool {
        self.track_search().can_fetch_more().unwrap_or(false)
    }
}

/// Library state with a handle to the runtime environment
#[allow(missing_debug_implementations)]
pub struct Library {
    handle: Handle,
    state_observables: StateObservables,
    state: State,
}

#[derive(Debug, Clone, Copy)]
pub enum OnTrackSearchStateChangedCompletionError {
    NotPending,
    AbortPendingAndRetry,
}

impl Library {
    #[must_use]
    pub fn new(handle: Handle, initial_settings: settings::State) -> Self {
        Self {
            handle,
            state_observables: StateObservables::new(initial_settings),
            state: Default::default(),
        }
    }

    #[must_use]
    pub const fn handle(&self) -> &Handle {
        &self.handle
    }

    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    #[must_use]
    pub fn read_lock_current_state(&self) -> CurrentState<'_> {
        self.state.read_lock_current(&self.state_observables)
    }

    #[must_use]
    pub fn read_lock_track_search_state(&self) -> track_search::StateRef<'_> {
        self.state_observables.track_search.read_lock()
    }

    #[must_use]
    pub fn subscribe_track_search_state_changed(&self) -> track_search::StateSubscriber {
        self.state_observables.track_search.subscribe_changed()
    }

    #[allow(clippy::must_use_candidate)]
    pub fn on_settings_state_changed(&mut self) -> bool {
        let new_music_dir = {
            let settings_state = self.state_observables.settings.read_lock();
            if settings_state.music_dir == self.state.music_dir {
                log::debug!(
                    "Music directory unchanged: {music_dir:?}",
                    music_dir = self.state.music_dir,
                );
                return false;
            }
            settings_state.music_dir.clone()
        };
        log::debug!(
            "Music directory changed: {old_music_dir:?} -> {new_music_dir:?}",
            old_music_dir = self.state.music_dir,
        );
        self.state.music_dir = new_music_dir;
        true
    }

    #[allow(clippy::must_use_candidate)]
    pub fn on_collection_state_changed(&mut self) -> bool {
        let new_state = {
            let new_state = self.state_observables.collection.read_lock();
            if *new_state == self.state.collection {
                log::debug!(
                    "Collection state unchanged: {old_state:?}",
                    old_state = self.state.collection,
                );
                return false;
            }
            new_state.clone()
        };
        log::debug!(
            "Collection state changed: {old_state:?} -> {new_state:?}",
            old_state = self.state.collection,
        );
        if self.state.pending_music_dir_sync_task.is_some()
            && !matches!(new_state, collection::State::Synchronizing { .. })
        {
            // The task will eventually finish.
            log::debug!("Resetting pending synchronize music directory task");
            self.state.pending_music_dir_sync_task = None;
        }
        self.state.collection = new_state;
        true
    }

    pub fn on_track_search_state_changed(
        &mut self,
    ) -> Option<(&track_search::Memo, track_search::MemoDiff)> {
        let (memo, memo_delta, memo_diff) = {
            let memo = match &mut self.state.track_search_memo {
                TrackSearchMemoState::Ready(memo) => memo,
                TrackSearchMemoState::Pending {
                    state_changed_again,
                    ..
                } => {
                    *state_changed_again = true;
                    return None;
                }
            };
            let (memo_delta, memo_diff) = {
                let state = self.state_observables.track_search.read_lock();
                state.update_memo_delta(memo)
            };
            let memo = std::mem::take(memo);
            (memo, memo_delta, memo_diff)
        };
        self.state.track_search_memo = TrackSearchMemoState::new_pending(memo, memo_delta);
        let TrackSearchMemoState::Pending { memo, .. } = &self.state.track_search_memo else {
            unreachable!();
        };
        Some((memo, memo_diff))
    }

    pub fn on_track_search_state_changed_complete_pending(
        &self,
    ) -> Result<
        (&track_search::Memo, &track_search::MemoDelta),
        OnTrackSearchStateChangedCompletionError,
    > {
        match &self.state.track_search_memo {
            TrackSearchMemoState::Ready(_) => {
                Err(OnTrackSearchStateChangedCompletionError::NotPending)
            }
            TrackSearchMemoState::Pending {
                memo,
                memo_delta,
                state_changed_again,
                pending_since,
            } => {
                log::debug!(
                    "Track search state changed pending completed after {elapsed_ms} ms",
                    elapsed_ms = pending_since.elapsed().as_secs_f64() * 1000.0
                );
                if *state_changed_again {
                    Err(OnTrackSearchStateChangedCompletionError::AbortPendingAndRetry)
                } else {
                    Ok((memo, memo_delta))
                }
            }
        }
    }

    #[allow(clippy::must_use_candidate)]
    pub fn on_track_search_state_changed_abort(&mut self) {
        let TrackSearchMemoState::Pending { memo, .. } = &mut self.state.track_search_memo else {
            unreachable!();
        };
        self.state.track_search_memo = TrackSearchMemoState::Ready(std::mem::take(memo));
    }

    #[allow(clippy::must_use_candidate)]
    pub fn on_track_search_state_changed_apply(&mut self) {
        let TrackSearchMemoState::Pending {
            memo,
            memo_delta,
            state_changed_again,
            pending_since: _,
        } = &mut self.state.track_search_memo
        else {
            unreachable!();
        };
        debug_assert!(!*state_changed_again);
        memo.apply_delta(std::mem::take(memo_delta));
        self.state.track_search_memo = TrackSearchMemoState::Ready(std::mem::take(memo));
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_update_music_dir(&self, music_dir: Option<&DirPath<'_>>) -> bool {
        if self
            .state_observables
            .settings
            .try_update_music_dir(music_dir)
        {
            log::info!("Music directory updated: {music_dir:?}");
            true
        } else {
            log::debug!("Music directory unchanged: {music_dir:?}");
            false
        }
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_reset_music_dir(&self) -> bool {
        self.try_update_music_dir(None)
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_reset_collection(&self) -> bool {
        self.state_observables.collection.try_reset()
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_spawn_music_dir_sync_task<E>(
        &mut self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
    ) -> bool
    where
        E: EventEmitter + Clone + 'static,
    {
        if let Some(sync_task) = self.state.pending_music_dir_sync_task.as_ref() {
            if sync_task.is_finished() {
                log::info!("Resetting synchronize music directory task after finished");
                self.state.pending_music_dir_sync_task = None;
            } else {
                log::info!("Synchronize music directory task still pending");
                return false;
            }
        }
        log::info!("Spawning synchronize music directory task");
        self.state.pending_music_dir_sync_task =
            SynchronizeVfsTask::try_spawn(rt, &self.handle, &self.state_observables.collection);
        let Some(task) = &self.state.pending_music_dir_sync_task else {
            return false;
        };
        rt.spawn({
            let event_emitter = event_emitter.clone();
            let mut subscriber = task.progress().clone();
            async move {
                loop {
                    let progress = subscriber.read_ack().clone();
                    if event_emitter
                        .emit_event(Event::MusicDirSyncProgress(progress))
                        .is_err()
                    {
                        break;
                    }
                    if subscriber.changed().await.is_err() {
                        break;
                    }
                }
            }
        });
        true
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_abort_pending_music_dir_sync_task(&mut self) -> bool {
        let pending_music_dir_sync_task = self.state.pending_music_dir_sync_task.take();
        let Some(synchronize_music_dir_task) = pending_music_dir_sync_task else {
            return false;
        };
        log::info!("Aborting synchronize music directory task");
        synchronize_music_dir_task.abort();
        true
    }

    #[allow(clippy::must_use_candidate)]
    #[allow(clippy::missing_panics_doc)] // Never panics
    pub fn try_view_music_dir_list<E>(
        &mut self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
        params: aoide::api::media::tracker::count_sources_in_directories::Params,
    ) -> bool
    where
        E: EventEmitter + Clone + 'static,
    {
        if !self.state.collection.is_ready() {
            log::debug!("Collection not ready");
            return false;
        }
        let collection_uid = self.state.collection.entity_uid().expect("Some").clone();
        let handle = self.handle.clone();
        let event_emitter = event_emitter.clone();
        rt.spawn(async move {
            let result = aoide::backend_embedded::media::tracker::count_sources_in_directories(
                handle.db_gatekeeper(),
                collection_uid.clone(),
                params.clone(),
            )
            .await
            .map_err(Into::into);
            let event = Event::MusicDirListResult {
                collection_uid,
                params,
                result,
            };
            event_emitter.emit_event(event).ok();
        });
        true
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_refresh_collection_from_db(&self, rt: &tokio::runtime::Handle) -> bool {
        let Some((task, continuation)) = self
            .state_observables
            .collection
            .try_refresh_from_db_task(&self.handle)
        else {
            return false;
        };
        log::debug!("Refreshing collection from DB");
        rt.spawn({
            let collection_state = Arc::clone(&self.state_observables.collection);
            async move {
                let result = task.await;
                collection_state.refresh_from_db_task_completed(result, continuation);
            }
        });
        true
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_search_tracks(&self, input: &str) -> bool {
        let filter = track_search::parse_filter_from_input(input);
        let mut params = aoide::api::track::search::Params {
            filter,
            ..track_search::default_params()
        };
        // Argument is consumed when updating succeeds
        log::debug!("Updating track search params: {params:?}");
        if !self
            .state_observables
            .track_search
            .try_update_params(&mut params)
        {
            log::debug!("Track search params not updated: {params:?}");
            return false;
        }
        true
    }

    #[allow(clippy::must_use_candidate)]
    pub fn try_spawn_fetch_more_track_search_results_task<E>(
        &self,
        tokio_rt: &tokio::runtime::Handle,
        event_emitter: &E,
    ) -> bool
    where
        E: EventEmitter + Clone + 'static,
    {
        let Some((task, continuation)) = self
            .state_observables
            .track_search
            .try_fetch_more_task(&self.handle, Some(track_search::DEFAULT_PREFETCH_LIMIT))
        else {
            return false;
        };
        log::debug!("Fetching more track search results");
        let event_emitter = event_emitter.clone();
        tokio_rt.spawn(async move {
            let result = task.await;
            event_emitter
                .emit_event(
                    track_search::Event::FetchMoreTaskCompleted {
                        result,
                        continuation,
                    }
                    .into(),
                )
                .ok();
        });
        true
    }

    #[allow(clippy::must_use_candidate)]
    pub fn on_fetch_more_track_search_results_task_completed(
        &self,
        result: track_search::FetchMoreResult,
        continuation: track_search::FetchMoreTaskContinuation,
    ) -> bool {
        self.state_observables
            .track_search
            .fetch_more_task_joined(result.into(), continuation)
    }

    /// Spawn reactive background tasks
    pub fn spawn_background_tasks(&self, tokio_rt: &tokio::runtime::Handle, settings_dir: PathBuf) {
        tokio_rt.spawn(settings::tasklet::on_state_changed_save_to_file(
            self.state_observables.settings.subscribe_changed(),
            settings_dir,
            |err| {
                log::error!("Failed to save settings to file: {err}");
            },
        ));
        tokio_rt.spawn(collection::tasklet::on_settings_state_changed(
            &self.state_observables.settings,
            Arc::downgrade(&self.state_observables.collection),
            Handle::downgrade(&self.handle),
            collection::CREATE_NEW_ENTITY_IF_NOT_FOUND,
            collection::NESTED_MUSIC_DIRS_STRATEGY,
        ));
        tokio_rt.spawn(track_search::tasklet::on_collection_state_changed(
            &self.state_observables.collection,
            Arc::downgrade(&self.state_observables.track_search),
        ));
        tokio_rt.spawn(track_search::tasklet::on_should_prefetch(
            &self.state_observables.track_search,
            Handle::downgrade(&self.handle),
            Some(track_search::DEFAULT_PREFETCH_LIMIT),
        ));
    }

    pub fn spawn_event_tasks<E>(&self, tokio_rt: &tokio::runtime::Handle, event_emitter: &E)
    where
        E: EventEmitter + Clone + 'static,
    {
        tokio_rt.spawn({
            let subscriber = self.state_observables.settings.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                settings::watch_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let subscriber = self.state_observables.collection.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                collection::watch_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let subscriber = self.state_observables.track_search.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                track_search::watch_state(subscriber, event_emitter).await;
            }
        });
    }
}
