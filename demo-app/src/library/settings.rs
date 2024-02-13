// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use discro::Subscriber;

use crate::NoReceiverForEvent;

use super::{LibraryEvent, LibraryEventEmitter};

// Re-exports
pub use aoide::desktop_app::settings::*;

pub type StateSubscriber = Subscriber<State>;

pub(super) async fn watch_state<E>(mut subscriber: StateSubscriber, event_emitter: E)
where
    E: LibraryEventEmitter,
{
    // The first event is always emitted immediately.
    loop {
        drop(subscriber.read_ack());
        if let Err(NoReceiverForEvent) =
            event_emitter.emit_event(LibraryEvent::SettingsStateChanged)
        {
            log::info!("Stop watching settings state after event receiver has been dropped");
            break;
        };
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching settings state after publisher has been dropped");
            break;
        }
    }
}
