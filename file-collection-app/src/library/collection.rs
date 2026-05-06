// SPDX-FileCopyrightText: Copyright (C) 2018-2026 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::convert::Infallible;

use discro::{Ref, Subscriber};

use super::EventEmitter;

// Re-exports
pub use aoide::collection::*;
pub use aoide::desktop_app::collection::*;

pub(super) const RESTORE_ENTITY_STRATEGY: RestoreEntityStrategy =
    RestoreEntityStrategy::LoadOrCreateNew;

pub(super) const NESTED_MUSIC_DIRS_STRATEGY: NestedMusicDirectoriesStrategy =
    NestedMusicDirectoriesStrategy::Permit;

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
    // The first event is always emitted immediately.
    log::debug!("Start watching collection state");
    let _unreachable: Infallible = loop {
        let event = on_state_changed(&mut subscriber);
        drop(event_emitter.emit_event(event.into()));

        log::debug!("Suspend watching collection state");
        if subscriber.changed().await.is_err() {
            log::debug!("Finish watching collection state after publisher has been dropped");
            return;
        }
        log::debug!("Resume watching collection state");
    };
}

#[must_use]
fn on_state_changed(subscriber: &mut StateSubscriber) -> Event {
    drop(subscriber.read_ack());
    Event::StateChanged
}
