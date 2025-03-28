// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::PathBuf, sync::Arc};

use anyhow::anyhow;
use aoide::{
    CollectionUid,
    desktop_app::{
        ActionEffect, Environment,
        collection::{State as CollectionState, SynchronizingVfsState, SynchronizingVfsTask},
    },
    media::content::ContentPath,
    util::fs::DirPath,
};
use discro::Ref;
use tokio::task::AbortHandle;

use crate::NoReceiverForEvent;

pub mod collection;
pub mod settings;
pub mod track_search;
pub mod ui;

#[derive(Debug)]
pub enum Event {
    Settings(settings::Event),
    Collection(collection::Event),
    TrackSearch(track_search::Event),
    MusicDirSyncProgress(
        Option<aoide::backend_embedded::batch::synchronize_collection_vfs::Progress>,
    ),
    MusicDirSyncFinished(Box<aoide::desktop_app::collection::SynchronizingVfsFinishedState>),
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
#[expect(missing_debug_implementations)]
pub struct SharedState {
    pub settings: Arc<settings::SharedState>,
    pub collection: Arc<collection::SharedState>,
    pub track_search: Arc<track_search::SharedState>,
}

impl SharedState {
    #[must_use]
    fn new(initial_settings: settings::State) -> Self {
        let settings = Arc::new(settings::SharedState::new(initial_settings));
        let collection = Arc::new(collection::SharedState::default());
        let track_search = Arc::new(track_search::SharedState::new(track_search::State::new(
            track_search::default_params(),
        )));
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
pub enum State {
    #[default]
    Idle,
    SynchronizingMusicDir {
        task: SynchronizingVfsTask,
    },
}

#[expect(missing_debug_implementations)]
pub struct CurrentState<'a> {
    pub settings: Ref<'a, settings::State>,
    pub collection: Ref<'a, collection::State>,
    pub track_search: Ref<'a, track_search::State>,
    pub library: &'a State,
}

impl CurrentState<'_> {
    #[must_use]
    pub fn could_reset_music_dir(&self) -> bool {
        self.settings.music_dir().is_some()
    }

    #[must_use]
    pub fn could_synchronize_music_dir_task(&self) -> bool {
        self.collection.is_ready()
    }

    #[must_use]
    pub fn could_abort_synchronize_music_dir_task(&self) -> bool {
        matches!(
            *self.collection,
            CollectionState::SynchronizingVfs {
                state: SynchronizingVfsState::Pending { .. },
                ..
            }
        )
    }

    #[must_use]
    pub fn could_view_music_dir_list(&self) -> bool {
        self.collection.is_ready() && !self.could_abort_synchronize_music_dir_task()
    }

    #[must_use]
    pub fn could_search_tracks(&self) -> bool {
        self.collection.is_ready() && self.track_search.pending_since().is_none()
    }

    #[must_use]
    pub fn could_fetch_more_track_search_results(&self) -> bool {
        self.track_search.can_fetch_more().unwrap_or(false)
    }
}

/// The library.
///
/// The runtime environment of the embedded backend and various stateful
/// components.
#[expect(missing_debug_implementations)]
pub struct Library {
    env: Arc<Environment>,
    shared_state: SharedState,
    state: State,
}

impl Library {
    #[must_use]
    pub fn new(env: Environment, initial_settings: settings::State) -> Self {
        Self {
            env: Arc::new(env),
            shared_state: SharedState::new(initial_settings),
            state: Default::default(),
        }
    }

    #[must_use]
    pub const fn env(&self) -> &Arc<Environment> {
        &self.env
    }

    #[must_use]
    pub const fn state(&self) -> &State {
        &self.state
    }

    #[must_use]
    pub fn read_current_state(&self) -> CurrentState<'_> {
        let Self {
            env: _,
            shared_state,
            state: library,
        } = self;
        let SharedState {
            settings,
            collection,
            track_search,
        } = shared_state;
        let settings = settings.read();
        let collection = collection.read();
        let track_search = track_search.read();
        CurrentState {
            settings,
            collection,
            track_search,
            library,
        }
    }

    #[must_use]
    pub fn read_settings_state(&self) -> settings::StateRef<'_> {
        self.shared_state.settings.read()
    }

    #[must_use]
    pub fn read_collection_state(&self) -> collection::StateRef<'_> {
        self.shared_state.collection.read()
    }

    #[must_use]
    pub fn read_track_search_state(&self) -> track_search::StateRef<'_> {
        self.shared_state.track_search.read()
    }

    #[must_use]
    pub fn subscribe_track_search_state_changed(&self) -> track_search::StateSubscriber {
        self.shared_state.track_search.subscribe_changed()
    }

    pub fn on_track_search_state_changed<'a>(
        &'a self,
        memo_state: &'a mut track_search::MemoState,
    ) -> Option<(&'a track_search::Memo, track_search::MemoDiff)> {
        memo_state.try_start_pending(&self.shared_state.track_search)
    }

    pub fn on_track_search_state_changed_pending_apply(
        &mut self,
        memo_state: &mut track_search::MemoState,
    ) {
        let track_search::MemoState::Pending {
            memo,
            memo_delta,
            state_changed_again,
            pending_since: _,
        } = memo_state
        else {
            unreachable!();
        };
        debug_assert!(!*state_changed_again);
        memo.apply_delta(std::mem::take(memo_delta));
        *memo_state = track_search::MemoState::Ready(std::mem::take(memo));
    }

    fn reset_idle(&mut self) -> ActionEffect {
        match std::mem::replace(&mut self.state, State::Idle) {
            State::Idle => ActionEffect::Unchanged,
            State::SynchronizingMusicDir { task } => {
                // Abort pending task.
                task.abort();
                ActionEffect::Changed
            }
        }
    }

    pub fn update_music_dir(&mut self, music_dir: Option<&DirPath<'_>>) -> ActionEffect {
        let mut effect = self.shared_state.settings.update_music_dir(music_dir);
        if !matches!(effect, ActionEffect::Unchanged) {
            effect += self.reset_idle();
        }
        effect
    }

    pub fn reset_music_dir(&mut self) -> ActionEffect {
        self.update_music_dir(None)
    }

    pub fn reset_collection(&mut self) -> ActionEffect {
        let mut effect = self.shared_state.collection.reset();
        if !matches!(effect, ActionEffect::Unchanged) {
            effect += self.reset_idle();
        }
        effect
    }

    pub fn sync_music_dir<E>(
        &mut self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
    ) -> (ActionEffect, anyhow::Result<()>)
    where
        E: EventEmitter + Clone + 'static,
    {
        if matches!(self.state, State::SynchronizingMusicDir { .. }) {
            let rejected = anyhow!("already/still pending");
            return (ActionEffect::Unchanged, Err(rejected));
        }
        let (mut effect, result) = self
            .shared_state
            .collection
            .spawn_synchronizing_vfs_task(rt, &self.env);
        let sync_music_dir_task = match result {
            Ok(task) => task,
            Err(err) => {
                return (effect, Err(err));
            }
        };

        let _monitor_progress = rt.spawn({
            let event_emitter = event_emitter.clone();
            let mut subscriber = sync_music_dir_task.progress().subscribe_changed();
            async move {
                loop {
                    log::debug!("Suspending sync_music_dir progress");
                    if subscriber.changed().await.is_err() {
                        // No publisher(s).
                        break;
                    }
                    log::debug!("Resuming sync_music_dir progress");
                    let progress = subscriber.read_ack().clone();
                    if event_emitter
                        .emit_event(Event::MusicDirSyncProgress(progress))
                        .is_err()
                    {
                        break;
                    }
                }
            }
        });

        let _emit_finished_state = rt.spawn({
            let rt = rt.clone();
            let env = Arc::clone(&self.env);
            let collection = Arc::clone(&self.shared_state.collection);
            let event_emitter = event_emitter.clone();
            async move {
                let (finish_effect, finish_result) =
                    collection.finish_synchronizing_vfs_task(&rt, &env).await;
                match finish_result {
                    Ok(finished_state) => {
                        debug_assert!(!matches!(finish_effect, ActionEffect::Unchanged));
                        log::debug!(
                            "Finished synchronization of music directory: {finished_state:?}"
                        );
                        let _ = event_emitter
                            .emit_event(Event::MusicDirSyncFinished(Box::new(finished_state)));
                    }
                    Err(err) => {
                        log::warn!("Failed to finish synchronization of music directory: {err:#}");
                    }
                }
            }
        });

        self.state = State::SynchronizingMusicDir {
            task: sync_music_dir_task,
        };
        effect += ActionEffect::Changed;

        (effect, Ok(()))
    }

    pub fn sync_music_dir_abort(&mut self) -> ActionEffect {
        let State::SynchronizingMusicDir { task } = &self.state else {
            log::info!("Not synchronizing music directory");
            return ActionEffect::Unchanged;
        };
        log::info!("Aborting synchronize music directory task");
        task.abort();
        ActionEffect::Changed
    }

    pub(crate) fn sync_music_dir_finished(&mut self) -> ActionEffect {
        debug_assert!(!matches!(
            *self.shared_state.collection.read(),
            CollectionState::SynchronizingVfs { .. }
        ));
        match std::mem::replace(&mut self.state, State::Idle) {
            State::Idle => ActionEffect::Unchanged,
            State::SynchronizingMusicDir { task } => {
                debug_assert!(task.is_finished());
                ActionEffect::Changed
            }
        }
    }

    pub fn view_music_dir_list<E>(
        &self,
        rt: &tokio::runtime::Handle,
        event_emitter: &E,
        params: aoide::api::media::tracker::count_sources_in_directories::Params,
    ) -> ActionEffect
    where
        E: EventEmitter + Clone + 'static,
    {
        let Some(collection_uid) = self.shared_state.collection.read().entity_uid().cloned() else {
            log::info!("No collection");
            return ActionEffect::Unchanged;
        };
        rt.spawn({
            let env = Arc::clone(self.env());
            let event_emitter = event_emitter.clone();
            async move {
                let result = aoide::backend_embedded::media::tracker::count_sources_in_directories(
                    env.db_gatekeeper(),
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
            }
        });
        ActionEffect::MaybeChanged
    }

    pub fn refresh_collection_from_db(
        &self,
        rt: &tokio::runtime::Handle,
    ) -> (ActionEffect, Option<AbortHandle>) {
        self.shared_state
            .collection
            .spawn_loading_from_database_task(rt, &self.env)
    }

    pub fn search_tracks(&self, input: &str) -> ActionEffect {
        let filter = track_search::parse_filter_from_input(input);
        let mut params = aoide::api::track::search::Params {
            filter,
            ..track_search::default_params()
        };
        // Argument is consumed when updating succeeds
        log::debug!("Updating track search params: {params:?}");
        self.shared_state.track_search.update_params(&mut params)
    }

    pub fn fetch_more_track_search_results(&self, rt: &tokio::runtime::Handle) -> ActionEffect {
        self.shared_state.track_search.spawn_fetching_more_task(
            rt,
            &self.env,
            Some(track_search::DEFAULT_PREFETCH_LIMIT),
        )
    }

    /// Spawn reactive background tasks
    pub fn spawn_background_tasks(&self, rt: &tokio::runtime::Handle, settings_dir: PathBuf) {
        rt.spawn(settings::tasklet::on_state_changed_save_to_file(
            &self.shared_state.settings.observe(),
            settings_dir,
            |err| {
                log::error!("Failed to save settings to file: {err}");
            },
        ));
        rt.spawn(collection::tasklet::on_settings_state_changed(
            rt.clone(),
            Arc::downgrade(&self.env),
            Arc::downgrade(&self.shared_state.collection),
            &self.shared_state.settings,
            collection::RESTORE_ENTITY_STRATEGY,
            collection::NESTED_MUSIC_DIRS_STRATEGY,
        ));
        rt.spawn(track_search::tasklet::on_collection_state_changed(
            &self.shared_state.collection,
            Arc::downgrade(&self.shared_state.track_search),
        ));
        rt.spawn(track_search::tasklet::on_should_prefetch(
            rt.clone(),
            Arc::downgrade(&self.env),
            &self.shared_state.track_search,
            Some(track_search::DEFAULT_PREFETCH_LIMIT),
        ));
    }

    pub fn spawn_event_tasks<E>(&self, rt: &tokio::runtime::Handle, event_emitter: &E)
    where
        E: EventEmitter + Clone + 'static,
    {
        rt.spawn({
            let subscriber = self.shared_state.settings.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                settings::watch_state(subscriber, event_emitter).await;
            }
        });
        rt.spawn({
            let subscriber = self.shared_state.collection.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                collection::watch_state(subscriber, event_emitter).await;
            }
        });
        rt.spawn({
            let subscriber = self.shared_state.track_search.subscribe_changed();
            let event_emitter = event_emitter.clone();
            async move {
                track_search::watch_state(subscriber, event_emitter).await;
            }
        });
    }
}
