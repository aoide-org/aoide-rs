// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

pub mod collection;
pub mod media;
pub mod prelude;

use prelude::*;

use std::{sync::Arc, time::Instant};

#[derive(Debug, Default)]
pub struct State {
    errors: Vec<anyhow::Error>,
    pub collection: collection::State,
    pub media_tracker: media::tracker::State,
}

impl State {
    pub fn errors(&self) -> &[anyhow::Error] {
        &self.errors
    }
}

#[derive(Debug)]
pub enum Action {
    Collection(collection::Action),
    MediaTracker(media::tracker::Action),
    EmitDeferredEvent {
        emit_not_before: Instant,
        event: Box<Event>,
    },
    Terminate,
}

impl From<collection::Action> for Action {
    fn from(from: collection::Action) -> Self {
        Self::Collection(from)
    }
}

impl From<media::tracker::Action> for Action {
    fn from(from: media::tracker::Action) -> Self {
        Self::MediaTracker(from)
    }
}

#[derive(Debug)]
pub enum Event {
    Collection(collection::Event),
    MediaTracker(media::tracker::Event),
    ErrorOccurred(anyhow::Error),
    EmitDeferred {
        emit_not_before: Instant,
        event: Box<Event>,
    },
    StateChanged,
    TerminateRequested,
}

#[derive(Debug)]
pub enum Intent {
    Collection(collection::Intent),
    //MediaTracker(media::tracker::Intent),
    EmitDeferred {
        emit_not_before: Instant,
        event: Box<Event>,
    },
    Terminate,
}

impl From<collection::Event> for Event {
    fn from(from: collection::Event) -> Self {
        Self::Collection(from)
    }
}

impl From<media::tracker::Event> for Event {
    fn from(from: media::tracker::Event) -> Self {
        Self::MediaTracker(from)
    }
}

pub type StateChangedFn = dyn FnMut(&State, &dyn EventEmitter<Event>) + Send;

pub async fn handle_events(
    env: Environment,
    event_channel: (EventSender<Event>, EventReceiver<Event>),
    initial_state: State,
    mut state_changed_fn: Box<StateChangedFn>,
) {
    let shared_env = Arc::new(env);
    let (event_tx, mut event_rx) = event_channel;
    let mut state = initial_state;
    while let Some(event) = event_rx.recv().await {
        log::debug!("Applying event: {:?}", event);
        let (applied_event, next_action) = apply_event(&mut state, event);
        if let Some(next_action) = next_action {
            if matches!(next_action, Action::Terminate) {
                break;
            }
            let shared_env = shared_env.clone();
            let event_tx = event_tx.clone();
            log::debug!("Dispatching next action: {:?}", next_action);
            tokio::spawn(dispatch_action(shared_env, event_tx, next_action));
        }
        match applied_event {
            AppliedEvent::Dropped => {
                log::warn!("Event dropped");
            }
            AppliedEvent::Accepted { state_changed } => {
                if state_changed {
                    log::debug!("State changed: {:?}", state);
                    tokio::task::block_in_place(|| state_changed_fn(&state, &event_tx));
                }
            }
        }
    }
}

fn apply_event(state: &mut State, event: Event) -> (AppliedEvent, Option<Action>) {
    match event {
        Event::ErrorOccurred(error)
        | Event::Collection(collection::Event::Effect(collection::Effect::ErrorOccurred(error)))
        | Event::MediaTracker(media::tracker::Event::Effect(
            media::tracker::Effect::ErrorOccurred(error),
        )) => {
            state.errors.push(error);
            (
                AppliedEvent::Accepted {
                    state_changed: true,
                },
                None,
            )
        }
        Event::Collection(event) => {
            let (applied_event, action) = collection::apply_event(&mut state.collection, event);
            (applied_event, action.map(Into::into))
        }
        Event::MediaTracker(event) => {
            let (applied_event, action) =
                media::tracker::apply_event(&mut state.media_tracker, event);
            (applied_event, action.map(Into::into))
        }
        Event::EmitDeferred {
            emit_not_before,
            event,
        } => (
            AppliedEvent::Accepted {
                state_changed: false,
            },
            Some(Action::EmitDeferredEvent {
                emit_not_before,
                event,
            }),
        ),
        Event::StateChanged => (
            AppliedEvent::Accepted {
                state_changed: true,
            },
            None,
        ),
        Event::TerminateRequested => {
            if state.media_tracker.is_idle() {
                (
                    AppliedEvent::Accepted {
                        state_changed: false,
                    },
                    Some(Action::Terminate),
                )
            } else {
                (AppliedEvent::Dropped, None)
            }
        }
    }
}

async fn dispatch_action(
    shared_env: Arc<Environment>,
    event_tx: EventSender<Event>,
    action: Action,
) {
    match action {
        Action::Collection(action) => {
            collection::dispatch_action(shared_env, event_tx, action).await;
        }
        Action::MediaTracker(action) => {
            media::tracker::dispatch_action(shared_env, event_tx, action).await;
        }
        Action::EmitDeferredEvent {
            emit_not_before,
            event,
        } => {
            tokio::time::sleep_until(emit_not_before.into()).await;
            emit_event(&event_tx, *event);
        }
        Action::Terminate => unreachable!(),
    }
}
