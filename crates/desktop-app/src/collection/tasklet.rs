// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::future::Future;

use discro::Subscriber;

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
