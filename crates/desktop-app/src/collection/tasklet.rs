// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    sync::{Arc, Weak},
};

use unnest::some_or_break;

use aoide_core::util::fs::DirPath;

use super::{NestedMusicDirectoriesStrategy, ObservableState, RestoreEntityStrategy};
use crate::{settings, Handle, StateUnchanged, WeakHandle};

async fn update_music_dir(
    settings_state: &settings::ObservableState,
    observable_state: &ObservableState,
    handle: Handle,
    music_dir: Option<DirPath<'static>>,
    collection_kind: Option<String>,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) {
    if let Err(StateUnchanged) = observable_state.update_music_dir(
        collection_kind.map(Into::into),
        music_dir,
        restore_entity,
        nested_music_directories,
    ) {
        return;
    }
    if let Ok((task, continuation)) = observable_state.refresh_from_db_task(&handle) {
        log::debug!("Refreshing from DB after updating music directory");
        let result = task.await;
        let _ = observable_state.refresh_from_db_task_completed(result, continuation);
    }
    // After succeeded read the actual music directory from the collection state
    // and feed it back into the settings state.
    let new_music_dir = {
        let state = observable_state.read();
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
    settings_state: &Arc<settings::ObservableState>,
    observable_state: Weak<ObservableState>,
    handle: WeakHandle,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) -> impl Future<Output = ()> + Send + 'static {
    let mut settings_state_sub = settings_state.subscribe_changed();
    let settings_state = Arc::downgrade(settings_state);
    async move {
        log::debug!("Starting on_settings_state_changed");
        loop {
            {
                let settings_state = some_or_break!(settings_state.upgrade());
                let observable_state = some_or_break!(observable_state.upgrade());
                let handle = some_or_break!(handle.upgrade());
                let (music_dir, collection_kind) = {
                    let settings_state = settings_state_sub.read_ack();
                    let music_dir = settings_state.music_dir().cloned().map(DirPath::into_owned);
                    let collection_kind = settings_state.collection_kind.clone();
                    (music_dir, collection_kind)
                };
                update_music_dir(
                    &settings_state,
                    &observable_state,
                    handle,
                    music_dir,
                    collection_kind,
                    restore_entity,
                    nested_music_directories,
                )
                .await;
            }
            log::debug!("Suspending on_settings_state_changed");
            if settings_state_sub.changed().await.is_err() {
                // Publisher disappeared
                break;
            }
            log::debug!("Resuming on_settings_state_changed");
        }
        log::debug!("Stopping on_settings_state_changed");
    }
}
