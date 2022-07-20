// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, path::PathBuf};

use discro::Subscriber;

use crate::fs::OwnedDirPath;

use super::State;

/// Listen for state changes.
///
/// The `on_changed` callback closure must return `true` to continue
/// listening and `false` to abort listening.
pub fn on_state_changed(
    mut state_sub: Subscriber<State>,
    mut on_changed: impl FnMut(&super::State) -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut value = state_sub.read().clone();
    async move {
        log::debug!("Starting on_state_changed");
        // Enforce initial update
        let mut value_changed = true;
        loop {
            #[allow(clippy::collapsible_if)] // suppress false positive warning
            if value_changed {
                if !on_changed(&value) {
                    // Consumer has rejected the notification
                    log::debug!("Aborting on_state_changed");
                    return;
                }
            }
            value_changed = false;
            if state_sub.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_state_changed");
                break;
            }
            let new_value = state_sub.read_ack();
            if value != *new_value {
                value = new_value.clone();
                value_changed = true;
            } else {
                log::debug!("state unchanged: {value:?}");
            }
        }
        log::debug!("Stopping on_state_changed");
    }
}

/// Save the settings after changed.
pub fn on_state_changed_saver(
    mut state_sub: Subscriber<State>,
    settings_dir: PathBuf,
    mut report_error: impl FnMut(anyhow::Error) + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial settings immediately before spawning the async task
    let mut old_settings = state_sub.read().clone();
    async move {
        log::debug!("Starting on_state_changed_saver");
        let mut settings_changed = false;
        loop {
            #[allow(clippy::collapsible_if)] // suppress false positive warning
            if settings_changed {
                log::debug!("Saving changed settings: {old_settings:?}");
                let new_settings = old_settings.clone();
                if let Err(err) = new_settings.save_spawn_blocking(settings_dir.clone()).await {
                    report_error(err);
                }
            }
            settings_changed = false;
            if state_sub.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_state_changed_saver");
                break;
            }
            let new_settings = state_sub.read_ack();
            if old_settings != *new_settings {
                old_settings = new_settings.clone();
                settings_changed = true;
            } else {
                log::debug!("Settings unchanged: {old_settings:?}");
            }
        }
    }
}

/// Listen for changes of the music directory.
///
/// The `on_changed` callback closure must return `true` to continue
/// listening and `false` to abort listening.
pub fn on_music_dir_changed(
    mut state_sub: Subscriber<State>,
    mut on_changed: impl FnMut(Option<&OwnedDirPath>) -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut value = state_sub.read().music_dir.clone();
    async move {
        log::debug!("Starting on_music_dir_changed");
        // Enforce initial update
        let mut value_changed = true;
        loop {
            #[allow(clippy::collapsible_if)] // suppress false positive warning
            if value_changed {
                if !on_changed(value.as_ref()) {
                    // Consumer has rejected the notification
                    log::debug!("Aborting on_music_dir_changed");
                    return;
                }
            }
            value_changed = false;
            if state_sub.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_music_dir_changed");
                break;
            }
            let settings = state_sub.read_ack();
            let new_value = settings.music_dir.as_ref();
            if value.as_ref() != new_value {
                value = new_value.cloned();
                value_changed = true;
            } else {
                log::debug!("Music directory unchanged: {value:?}");
            }
        }
        log::debug!("Stopping on_music_dir_changed");
    }
}
