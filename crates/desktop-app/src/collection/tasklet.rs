// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    sync::{Arc, Weak},
};

use unnest::some_or_break;

use aoide_core::util::fs::DirPath;

use super::{
    NestedMusicDirectoriesStrategy, RestoreEntityStrategy, RestoringFromMusicDirectoryState,
    SharedState, State,
};
use crate::{collection::SpawnRestoringFromMusicDirectoryTaskReaction, settings, Environment};

#[allow(clippy::too_many_arguments)] // TODO
async fn update_music_dir(
    rt: &tokio::runtime::Handle,
    env: &Arc<Environment>,
    state: &SharedState,
    settings: &settings::SharedState,
    music_dir: Option<DirPath<'static>>,
    collection_kind: Option<String>,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) {
    let Some(music_dir) = music_dir else {
        log::debug!("Resetting music directory");
        let _ = state.reset();
        return;
    };

    {
        let task = match state.spawn_restoring_from_music_directory_task(
            rt,
            env,
            collection_kind.map(Into::into),
            music_dir,
            restore_entity,
            nested_music_directories,
        ) {
            Ok(SpawnRestoringFromMusicDirectoryTaskReaction::SpawnedAndChanged(task)) => task,
            Ok(SpawnRestoringFromMusicDirectoryTaskReaction::Unchanged) => {
                // Nothing to do.
                return;
            }
            Err(err) => {
                log::warn!("Failed to restore from music directory: {err:#}");
                return;
            }
        };
        let mut subscriber = state.subscribe_changed();
        while matches!(
            *subscriber.read_ack(),
            State::RestoringFromMusicDirectory {
                state: RestoringFromMusicDirectoryState::Pending { .. },
                ..
            }
        ) {
            if subscriber.changed().await.is_err() {
                // No publisher(s).
                break;
            }
        }
        debug_assert!(task.is_finished());
    }

    // After succeeded read the actual music directory from the collection state
    // and feed it back into the settings state.
    let new_music_dir = {
        let state = state.read();
        if !state.is_ready() {
            return;
        }
        state.music_dir().map(DirPath::into_owned)
    };
    if let Some(new_music_dir) = &new_music_dir {
        log::info!(
            "Updating music directory in settings: {new_music_dir}",
            new_music_dir = new_music_dir.display()
        );
    } else {
        log::info!("Resetting music directory in settings");
    }
    let _ = settings.update_music_dir(new_music_dir.as_ref());
}

pub fn on_settings_state_changed(
    rt: tokio::runtime::Handle,
    env: Weak<Environment>,
    state: Weak<SharedState>,
    settings: &Arc<settings::SharedState>,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) -> impl Future<Output = ()> + Send + 'static + use<> {
    let mut settings_subscriber = settings.subscribe_changed();
    let settings = Arc::downgrade(settings);
    async move {
        log::debug!("Starting on_settings_state_changed");
        loop {
            log::debug!("Suspending on_settings_state_changed");
            if settings_subscriber.changed().await.is_err() {
                // No publisher(s).
                break;
            }
            log::debug!("Resuming on_settings_state_changed");

            let env = some_or_break!(env.upgrade());
            let state = some_or_break!(state.upgrade());
            let settings = some_or_break!(settings.upgrade());
            let (music_dir, collection_kind) = {
                let settings = settings_subscriber.read_ack();
                let music_dir = settings.music_dir().cloned().map(DirPath::into_owned);
                let collection_kind = settings.collection_kind.clone();
                (music_dir, collection_kind)
            };
            update_music_dir(
                &rt,
                &env,
                &state,
                &settings,
                music_dir,
                collection_kind,
                restore_entity,
                nested_music_directories,
            )
            .await;
        }
    }
}
