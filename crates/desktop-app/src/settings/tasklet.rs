// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, path::PathBuf};

use discro::{tasklet::OnChanged, Observer};

use aoide_core::util::fs::DirPath;

use super::State;

/// Save the settings after changed.
///
/// The current settings at the time of invocation are not saved.
pub fn on_state_changed_save_to_file(
    this: &Observer<State>,
    settings_dir: PathBuf,
    mut report_error: impl FnMut(anyhow::Error) + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read and acknowledge the initial settings immediately before spawning
    // the async task. These are supposed to be saved already. Only subsequent
    // changes will be captured, which might occur already while spawning the task.
    // Otherwise when reading the initial settings later within the spawned task
    // all intermediate changes would slip through unnoticed!
    let mut subscriber = this.subscribe_changed();
    let mut settings = subscriber.read_ack().clone();
    async move {
        log::debug!("Starting on_state_changed_save_to_file");
        loop {
            log::debug!("Suspending on_state_changed_save_to_file");
            if subscriber.changed().await.is_err() {
                // No publisher(s).
                break;
            }
            log::debug!("Resuming on_state_changed_save_to_file");

            {
                let new_settings = subscriber.read_ack();
                if settings == *new_settings {
                    log::debug!("Settings unchanged: {settings:?}");
                    continue;
                }
                settings = new_settings.clone();
            }

            log::info!("Saving changed settings: {settings:?}");
            let save_settings = settings.clone();
            if let Err(err) = save_settings
                .save_spawn_blocking(settings_dir.clone())
                .await
            {
                report_error(err);
            }
        }
    }
}

/// Listen for changes of the music directory.
pub fn on_music_dir_changed(
    this: &Observer<State>,
    mut on_changed: impl FnMut(Option<&DirPath<'_>>) -> OnChanged + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut subscriber = this.subscribe();
    let mut value = subscriber
        .read_ack()
        .music_dir()
        .cloned()
        .map(DirPath::into_owned);
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
                        return;
                    }
                }
                value_changed = false;
            }

            log::debug!("Suspending on_music_dir_changed");
            if subscriber.changed().await.is_err() {
                // No publisher(s).
                break;
            }
            log::debug!("Resuming on_music_dir_changed");

            let settings = subscriber.read_ack();
            let new_value = settings.music_dir();
            if value.as_ref() == new_value {
                continue;
            }
            // Only clone the new value if it differs from the current value.
            value = new_value.cloned().map(DirPath::into_owned);
            value_changed = true;
        }
    }
}
