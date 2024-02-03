// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

use aoide::{
    api::media::source::ResolveUrlFromContentPath,
    desktop_app::{fs::DirPath, Handle, ObservableReader},
};

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
pub enum LibraryNotification {
    SettingsStateChanged(settings::State),
    CollectionStateChanged,
    TrackSearchStateChanged,
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
    track_search: Arc<track_search::ObservableState>,
}

impl LibraryState {
    #[must_use]
    pub fn new(initial_settings: settings::State) -> Self {
        let initial_track_search = track_search::State::new(default_track_search_params());
        Self {
            settings: Arc::new(settings::ObservableState::new(initial_settings)),
            collection: Arc::default(),
            track_search: Arc::new(track_search::ObservableState::new(initial_track_search)),
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
}

/// Library state with a handle to the runtime environment
#[allow(missing_debug_implementations)]
pub struct Library {
    handle: Handle,
    state: LibraryState,
    pending_rescan_collection_task: Option<collection::RescanTask>,
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

    pub fn update_music_directory(&self, music_dir: Option<&DirPath<'_>>) {
        if self.state.settings.update_music_dir(music_dir) {
            log::info!("Music directory updated: {music_dir:?}");
        } else {
            log::debug!("Music directory unchanged: {music_dir:?}");
        }
    }

    pub fn reset_music_directory(&self) {
        self.update_music_directory(None);
    }

    pub fn reset_collection(&self) {
        self.state.collection.reset();
    }

    pub fn spawn_rescan_collection_task(&mut self, rt: &tokio::runtime::Handle) -> bool {
        if let Some(rescan_collection_task) = self.pending_rescan_collection_task.as_ref() {
            if rescan_collection_task.is_finished() {
                log::info!("Resetting finished rescan collection task");
                self.pending_rescan_collection_task = None;
            } else {
                log::info!("Rescan collection still pending");
                return false;
            }
        }
        log::info!("Spawning rescan collection task");
        let handle = self.handle.clone();
        let collection = Arc::clone(&self.state.collection);
        let rescan_collection_task = collection::RescanTask::spawn(rt, handle, collection);
        self.pending_rescan_collection_task = Some(rescan_collection_task);
        true
    }

    pub const fn pending_rescan_collection_task(&self) -> Option<&collection::RescanTask> {
        self.pending_rescan_collection_task.as_ref()
    }

    pub fn abort_pending_rescan_collection_task(&mut self) -> Option<collection::RescanTask> {
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
        if self.state.track_search.update_params(&mut params) {
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
            &self.state.settings,
            Arc::downgrade(&self.state.collection),
            Handle::downgrade(&self.handle),
            CREATE_NEW_COLLECTION_ENTITY_IF_NOT_FOUND,
            NESTED_MUSIC_DIRS,
            |err| {
                log::error!("Failed to update collection after settings state changed: {err}");
            },
        ));
        tokio_rt.spawn(track_search::tasklet::on_collection_changed(
            &self.state.collection,
            Arc::downgrade(&self.state.track_search),
        ));
        tokio_rt.spawn(track_search::tasklet::on_should_prefetch(
            &self.state.track_search,
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
            let subscriber = self.state().settings.subscribe_changed();
            async move {
                settings::watch_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let subscriber = self.state().collection.subscribe_changed();
            async move {
                collection::watch_state(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let subscriber = self.state().track_search.subscribe_changed();
            async move {
                track_search::watch_state(subscriber, event_emitter).await;
            }
        });
    }
}
