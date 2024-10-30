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
use crate::{settings, Environment};

#[allow(clippy::too_many_arguments)] // TODO
async fn update_music_dir(
    rt: &tokio::runtime::Handle,
    env: &Arc<Environment>,
    collection_state: &SharedState,
    settings_state: &settings::SharedState,
    music_dir: Option<DirPath<'static>>,
    collection_kind: Option<String>,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) {
    let Some(music_dir) = music_dir else {
        let _ = collection_state.reset();
        return;
    };

    if !matches!(
        collection_state.spawn_restoring_from_music_directory_task(
            rt,
            env,
            collection_kind.map(Into::into),
            music_dir,
            restore_entity,
            nested_music_directories,
        ),
        (_, Ok(Some(_)))
    ) {
        return;
    }

    let mut subscriber = collection_state.subscribe_changed();
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

    // After succeeded read the actual music directory from the collection state
    // and feed it back into the settings state.
    let new_music_dir = {
        let state = collection_state.read();
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
        log::info!("Resetting music directory in settings",);
    }
    let _ = settings_state.update_music_dir(new_music_dir.as_ref());
}

pub fn on_settings_state_changed(
    rt: tokio::runtime::Handle,
    env: Weak<Environment>,
    settings_state: &Arc<settings::SharedState>,
    collection_state: Weak<SharedState>,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) -> impl Future<Output = ()> + Send + 'static {
    let mut settings_state_sub = settings_state.subscribe_changed();
    let settings_state = Arc::downgrade(settings_state);
    async move {
        log::debug!("Starting on_settings_state_changed");
        loop {
            log::debug!("Suspending on_settings_state_changed");
            if settings_state_sub.changed().await.is_err() {
                // No publisher(s).
                break;
            }
            log::debug!("Resuming on_settings_state_changed");
            let env = some_or_break!(env.upgrade());
            let collection_state = some_or_break!(collection_state.upgrade());
            let settings_state = some_or_break!(settings_state.upgrade());
            let (music_dir, collection_kind) = {
                let settings_state = settings_state_sub.read_ack();
                let music_dir = settings_state.music_dir().cloned().map(DirPath::into_owned);
                let collection_kind = settings_state.collection_kind.clone();
                (music_dir, collection_kind)
            };
            update_music_dir(
                &rt,
                &env,
                &collection_state,
                &settings_state,
                music_dir,
                collection_kind,
                restore_entity,
                nested_music_directories,
            )
            .await;
        }
    }
}
