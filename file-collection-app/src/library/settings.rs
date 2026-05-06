// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use discro::{Ref, Subscriber};

use super::EventEmitter;

// Re-exports
pub use aoide::desktop_app::settings::*;

#[derive(Debug)]
pub enum Event {
    StateChanged,
}

pub type StateRef<'a> = Ref<'a, State>;
pub type StateSubscriber = Subscriber<State>;

pub(super) async fn watch_state<E>(mut subscriber: StateSubscriber, event_emitter: E)
where
    E: EventEmitter,
{
    log::debug!("Start watching settings state");
    // Emit the first event immediately.
    loop {
        let event = on_state_changed(&mut subscriber);
        drop(event_emitter.emit_event(event.into()));

        log::debug!("Suspend watching settings state");
        if subscriber.changed().await.is_err() {
            log::info!("Finish watching settings state");
            break;
        }
        log::debug!("Resume watching settings state");
    }
}

#[must_use]
fn on_state_changed(subscriber: &mut StateSubscriber) -> Event {
    drop(subscriber.read_ack());
    Event::StateChanged
}
