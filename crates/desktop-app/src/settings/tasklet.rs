// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, path::PathBuf};

use discro::Subscriber;

use crate::fs::OwnedDirPath;

use super::Settings;

/// Save the settings after changed.
pub fn on_state_changed_saver(
    mut settings_sub: Subscriber<Settings>,
    settings_dir: PathBuf,
    mut report_save_error: impl FnMut(anyhow::Error) + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial settings immediately before spawning the async task
    let mut old_settings = settings_sub.read().to_owned();
    async move {
        log::debug!("Starting on_state_changed_saver");
        while settings_sub.changed().await.is_ok() {
            let new_settings = settings_sub.read_ack().to_owned();
            if old_settings != new_settings {
                log::debug!("Saving changed settings: {old_settings:?} -> {new_settings:?}");
                old_settings = new_settings.clone();
                if let Err(err) = new_settings.save_spawn_blocking(settings_dir.clone()).await {
                    report_save_error(err);
                }
            }
        }
        log::debug!("Stopping on_state_changed_saver");
    }
}

/// Listen for changes of the music directory.
///
/// The `on_changed` callback closure must return `true` to continue
/// listening and `false` to abort listening.
pub fn on_music_dir_changed(
    mut settings_sub: Subscriber<Settings>,
    mut on_changed: impl FnMut(Option<&OwnedDirPath>) -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut value = settings_sub.read_ack().music_dir.clone();
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
            if settings_sub.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_music_dir_changed");
                break;
            }
            let settings = settings_sub.read_ack();
            let new_value = settings.music_dir.as_ref();
            if value.as_ref() != new_value {
                value = new_value.cloned();
                value_changed = true;
            }
        }
        log::debug!("Stopping on_music_dir_changed");
    }
}
