// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, path::PathBuf, sync::Arc};

use aoide::{
    api::media::source::ResolveUrlFromContentPath,
    desktop_app::{collection, settings, track, Handle},
};
use discro::tasklet::OnChanged;

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

#[derive(Debug, Clone)]
pub enum LibraryNotification {
    MusicDirChanged(Option<PathBuf>),
    CollectionChanged(Option<aoide::collection::Entity>),
}

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
#[derive(Clone)]
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
                log::error!("Failed to update collection after music directory changed: {err}");
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
            settings::tasklet::on_music_dir_changed(subscriber, move |music_dir| {
                let Some(event_emitter) = event_emitter.upgrade() else {
                    return OnChanged::Abort;
                };
                let music_dir = music_dir.map(|dir_path| dir_path.clone().into_owned().into());
                event_emitter.emit_notification(LibraryNotification::MusicDirChanged(music_dir));
                OnChanged::Continue
            })
        });
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let initial_state_tag = self.state().collection().read().state_tag();
            let observable_state = Arc::downgrade(self.state().collection());
            let subscriber = self.state().collection().subscribe_changed();
            let on_changed = move |_: collection::StateTag| {
                let Some(event_emitter) = event_emitter.upgrade() else {
                    return OnChanged::Abort;
                };
                let Some(observable_state) = observable_state.upgrade() else {
                    return OnChanged::Abort;
                };
                let collection = observable_state.read().entity().cloned();
                event_emitter.emit_notification(LibraryNotification::CollectionChanged(collection));
                OnChanged::Continue
            };
            // Invoke `on_changed()` with the initial state tag to emit an initial notification.
            on_changed(initial_state_tag);
            collection::tasklet::on_state_tag_changed(subscriber, on_changed)
        });
        // ...TODO...
    }
}
