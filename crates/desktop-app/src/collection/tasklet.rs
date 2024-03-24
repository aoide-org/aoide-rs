// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    future::Future,
    sync::{Arc, Weak},
};

use unnest::some_or_break;

use super::{NestedMusicDirectoriesStrategy, ObservableState, RestoreEntityStrategy};
use crate::{fs::DirPath, settings, Handle, WeakHandle};

async fn try_update_music_dir(
    settings_state: &settings::ObservableState,
    observable_state: &ObservableState,
    handle: Handle,
    music_dir: Option<DirPath<'static>>,
    collection_kind: Option<String>,
    restore_entity: RestoreEntityStrategy,
    nested_music_directories: NestedMusicDirectoriesStrategy,
) -> bool {
    if !observable_state.try_update_music_dir(
        collection_kind.map(Into::into),
        music_dir,
        restore_entity,
        nested_music_directories,
    ) {
        // Unchanged
        return false;
    }
    if let Some((task, continuation)) = observable_state.try_refresh_from_db_task(&handle) {
        log::debug!("Refreshing from DB after updating music directory");
        let result = task.await;
        observable_state.refresh_from_db_task_completed(result, continuation);
    }
    // After succeeded read the actual music directory from the collection state
    // and feed it back into the settings state.
    let new_music_dir = {
        let state = observable_state.read();
        if !state.is_ready() {
            return false;
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
    settings_state.try_update_music_dir(new_music_dir.as_ref())
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
                    let music_dir = settings_state.music_dir.clone();
                    let collection_kind = settings_state.collection_kind.clone();
                    (music_dir, collection_kind)
                };
                try_update_music_dir(
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
