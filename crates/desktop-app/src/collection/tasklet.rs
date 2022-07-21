// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Arc};

use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;
use discro::Subscriber;

use crate::{fs::DirPath, settings};

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

/// Listen for changes of [`State::is_ready()`].
///
/// The `on_changed` callback closure must return `true` to continue
/// listening and `false` to abort listening.
pub fn on_is_ready_changed(
    mut state_sub: Subscriber<State>,
    mut on_changed: impl FnMut(bool) -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut value = state_sub.read().is_ready();
    async move {
        log::debug!("Starting on_is_ready_changed");
        // Enforce initial update
        let mut value_changed = true;
        loop {
            #[allow(clippy::collapsible_if)] // suppress false positive warning
            if value_changed {
                if !on_changed(value) {
                    // Consumer has rejected the notification
                    log::debug!("Aborting on_is_ready_changed");
                    return;
                }
            }
            value_changed = false;
            if state_sub.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_is_ready_changed");
                break;
            }
            let new_value = state_sub.read_ack().is_ready();
            if value != new_value {
                value = new_value;
                value_changed = true;
            } else {
                log::debug!("is_ready unchanged: {value}");
            }
        }
        log::debug!("Stopping on_is_ready_changed");
    }
}

pub async fn on_music_dir_changed_updater(
    db_gatekeeper: Arc<Gatekeeper>,
    settings_state: Arc<settings::ObservableState>,
    collection_state: Arc<super::ObservableState>,
    mut report_error: impl FnMut(anyhow::Error) + Send + 'static,
) {
    log::debug!("Starting on_music_dir_changed_updater");
    let mut settings_state_sub = settings_state.subscribe();
    while settings_state_sub.changed().await.is_ok() {
        let (music_dir, collection_kind) = {
            let settings_state = settings_state_sub.read();
            let music_dir = settings_state.music_dir.clone();
            let collection_kind = settings_state.collection_kind.clone();
            (music_dir, collection_kind)
        };
        if let Err(err) = collection_state
            .update_music_dir(&db_gatekeeper, music_dir, collection_kind.map(Into::into))
            .await
        {
            report_error(err);
            // Reset the music directory in the settings state. This will
            // reset the collection state subsequently.
            settings_state.modify(|settings| settings.update_music_dir(None));
        } else {
            // Get the actual music directory from the collection state
            // and feed it back into the settings state.
            let music_dir = collection_state.read().music_dir().map(DirPath::into_owned);
            settings_state.modify(|settings| settings.update_music_dir(music_dir.as_ref()));
        }
    }
    log::debug!("Stopping on_music_dir_changed_updater");
}
