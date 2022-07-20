// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{future::Future, sync::Arc};

use discro::Subscriber;

use crate::{collection, fs::DirPath, settings, Environment};

use super::State;

/// Listen for changes of [`State::is_pending()`].
///
/// The `on_changed` callback closure must return `true` to continue
/// listening and `false` to abort listening.
pub fn on_is_pending_changed(
    mut state_sub: Subscriber<State>,
    mut on_changed: impl FnMut(bool) -> bool + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    // Read the initial value immediately before spawning the async task
    let mut value = state_sub.read_ack().is_pending();
    async move {
        log::debug!("Starting on_is_pending_changed");
        // Enforce initial update
        let mut value_changed = true;
        loop {
            #[allow(clippy::collapsible_if)] // suppress false positive warning
            if value_changed {
                if !on_changed(value) {
                    // Consumer has rejected the notification
                    log::debug!("Aborting on_is_pending_changed");
                    return;
                }
            }
            value_changed = false;
            if state_sub.changed().await.is_err() {
                // Publisher has disappeared
                log::debug!("Aborting on_is_pending_changed");
                break;
            }
            let new_value = state_sub.read_ack().is_pending();
            if value != new_value {
                value = new_value;
                value_changed = true;
            }
        }
        log::debug!("Stopping on_is_pending_changed");
    }
}

pub fn on_music_dir_changed_updater(
    environment: &Environment,
    settings_state: Arc<settings::ObservableState>,
    collection_state: Arc<super::ObservableState>,
    mut report_error: impl FnMut(&str, &anyhow::Error) + Send + 'static,
) -> impl Future<Output = ()> + Send + 'static {
    let db_gatekeeper = Arc::clone(environment.db_gatekeeper());
    async move {
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
                .update_music_dir(
                    &db_gatekeeper,
                    music_dir.as_deref(),
                    collection_kind.map(Into::into),
                )
                .await
            {
                report_error(
                    "Failed to update collection after music directory changed",
                    &err,
                );
                collection_state.modify(collection::State::reset);
            } else {
                let music_dir = collection_state.read().music_dir().map(DirPath::into_owned);
                settings_state.modify(|settings| settings.update_music_dir(music_dir.as_ref()));
            }
        }
        log::debug!("Stopping on_music_dir_changed_updater");
    }
}
