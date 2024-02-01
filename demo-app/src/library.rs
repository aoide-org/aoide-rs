// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    num::NonZeroUsize,
    path::PathBuf,
    sync::{Arc, Weak},
};

use aoide::{
    api::media::source::ResolveUrlFromContentPath,
    desktop_app::{collection, settings, track, Handle},
};
use discro::Subscriber;

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
    MusicDirectoryChanged(Option<PathBuf>),
    CollectionEntityChanged(Option<aoide::collection::Entity>),
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
            async move {
                watch_music_directory(subscriber, event_emitter).await;
            }
        });
        tokio_rt.spawn({
            let event_emitter = Arc::downgrade(event_emitter);
            let subscriber = self.state().collection().subscribe_changed();
            async move {
                watch_collection_entity(subscriber, event_emitter).await;
            }
        });
        // ...TODO...
    }
}

async fn watch_music_directory<E>(
    mut subscriber: Subscriber<settings::State>,
    event_emitter: Weak<E>,
) where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    let mut music_dir = subscriber.read_ack().music_dir.clone();
    'outer: loop {
        {
            let Some(event_emitter) = event_emitter.upgrade() else {
                log::info!("Stop watching music directory after event emitter has been dropped");
                break 'outer;
            };
            // No locks are held here, so we can safely call `emit_notification()`.
            event_emitter.emit_notification(LibraryNotification::MusicDirectoryChanged(
                music_dir.clone().map(Into::into),
            ));
        }
        'inner: loop {
            if subscriber.changed().await.is_err() {
                log::info!("Stop watching music directory after publisher has been dropped");
                break 'outer;
            }
            let new_music_dir = &subscriber.read_ack().music_dir;
            if music_dir != *new_music_dir {
                music_dir = new_music_dir.clone().map(Into::into);
                // Break out of the inner loop to emit a notification AFTER releasing the lock.
                break 'inner;
            }
        }
    }
}

async fn watch_collection_entity<E>(
    mut subscriber: Subscriber<collection::State>,
    event_emitter: Weak<E>,
) where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    let mut entity = subscriber.read_ack().entity().cloned();
    'outer: loop {
        {
            let Some(event_emitter) = event_emitter.upgrade() else {
                log::info!("Stop watching collection entity after event emitter has been dropped");
                break 'outer;
            };
            // No locks are held here, so we can safely call `emit_notification()`.
            event_emitter
                .emit_notification(LibraryNotification::CollectionEntityChanged(entity.clone()));
        }
        'inner: loop {
            if subscriber.changed().await.is_err() {
                log::info!("Stop watching collection entity after publisher has been dropped");
                break 'outer;
            }
            let state = subscriber.read_ack();
            let new_entity = state.entity();
            if entity.as_ref() != new_entity {
                entity = new_entity.cloned();
                // Break out of the inner loop to emit a notification AFTER releasing the lock.
                break 'inner;
            }
        }
    }
}
