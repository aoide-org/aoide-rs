// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, path::PathBuf};

use discro::{tasklet::OnChanged, Subscriber};

use super::State;
use crate::fs::DirPath;

/// Save the settings after changed.
pub fn on_state_changed_save_to_file(
    mut subscriber: Subscriber<State>,
    settings_dir: PathBuf,
    mut report_error: impl FnMut(anyhow::Error) + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial settings immediately before spawning the async task.
    // These are supposed to be saved already. Only subsequent changes will
    // be noticed, which might occur already while spawning the task. Otherwise
    // when reading the initial settings later within the spawned task all
    // intermediate changes would slip through unnoticed!
    let mut old_settings = subscriber.read().clone();
    async move {
        log::debug!("Starting on_state_changed_save_to_file");
        let mut settings_changed = false;
        loop {
            if settings_changed {
                log::debug!("Saving changed settings: {old_settings:?}");
                let new_settings = old_settings.clone();
                if let Err(err) = new_settings.save_spawn_blocking(settings_dir.clone()).await {
                    report_error(err);
                }
            }
            settings_changed = false;
            if subscriber.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_state_changed_save_to_file");
                break;
            }
            let new_settings = subscriber.read_ack();
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
pub fn on_music_dir_changed(
    mut subscriber: Subscriber<State>,
    mut on_changed: impl FnMut(Option<&DirPath<'_>>) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut value = subscriber.read().music_dir.clone();
    async move {
        log::debug!("Starting on_music_dir_changed");
        // Enforce initial update
        let mut value_changed = true;
        loop {
            if value_changed {
                log::debug!("on_music_dir_changed({value:?})");
                match on_changed(value.as_ref()) {
                    OnChanged::Continue => (),
                    OnChanged::Abort => {
                        // Consumer has rejected the notification
                        log::debug!("Aborting on_music_dir_changed");
                        return;
                    }
                }
            }
            value_changed = false;
            if subscriber.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_music_dir_changed");
                break;
            }
            let settings = subscriber.read_ack();
            let new_value = settings.music_dir.as_ref();
            if value.as_ref() != new_value {
                value = new_value.cloned();
                value_changed = true;
            }
        }
        log::debug!("Stopping on_music_dir_changed");
    }
}
