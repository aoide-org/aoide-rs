// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use discro::{Ref, Subscriber};

use crate::NoReceiverForEvent;

use super::EventEmitter;

// Re-exports
pub use aoide::collection::*;
pub use aoide::desktop_app::collection::*;

pub(super) const CREATE_NEW_ENTITY_IF_NOT_FOUND: bool = true;

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
    loop {
        drop(subscriber.read_ack());
        if let Err(NoReceiverForEvent) = event_emitter.emit_event(Event::StateChanged.into()) {
            log::info!("Stop watching collection state after event receiver has been dropped");
            break;
        };
        if subscriber.changed().await.is_err() {
            log::info!("Stop watching collection state after publisher has been dropped");
            break;
        }
    }
}
