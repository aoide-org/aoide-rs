// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Weak;

use discro::Subscriber;

use super::{LibraryEventEmitter, LibraryNotification};

// Re-exports
pub use aoide::desktop_app::settings::*;

pub(super) async fn watch_state<E>(mut subscriber: Subscriber<State>, event_emitter: Weak<E>)
where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        let Some(event_emitter) = event_emitter.upgrade() else {
            log::info!("Stop watching settings state after event emitter has been dropped");
            break;
        };
        // The lock is released immediately after cloning the state.
        let state = subscriber.read_ack().clone();
        event_emitter.emit_notification(LibraryNotification::SettingsStateChanged(state.clone()));
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching settings state after publisher has been dropped");
            break;
        }
    }
}
