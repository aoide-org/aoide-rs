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

use bytes::Bytes;
use prelude::*;
use reqwest::Response;

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
    Intent(Intent),
    Effect(Effect),
    CollectionEvent(collection::Event),
    MediaTrackerEvent(media::tracker::Event),
}

#[derive(Debug)]
pub enum Intent {
    EmitDeferredEvent {
        emit_not_before: Instant,
        event: Box<Event>,
    },
    RenderState,
    Terminate,
}

impl From<Intent> for Event {
    fn from(intent: Intent) -> Self {
        Self::Intent(intent)
    }
}

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
}

impl From<Effect> for Event {
    fn from(effect: Effect) -> Self {
        Self::Effect(effect)
    }
}

impl From<collection::Event> for Event {
    fn from(from: collection::Event) -> Self {
        Self::CollectionEvent(from)
    }
}

impl From<media::tracker::Event> for Event {
    fn from(from: media::tracker::Event) -> Self {
        Self::MediaTrackerEvent(from)
    }
}

pub type RenderStateFn = dyn FnMut(&State, &dyn EventEmitter<Event>) + Send;

pub async fn handle_events(
    env: Environment,
    event_channel: (EventSender<Event>, EventReceiver<Event>),
    initial_state: State,
    mut render_state_fn: Box<RenderStateFn>,
) {
    let shared_env = Arc::new(env);
    let (event_tx, mut event_rx) = event_channel;
    let mut state = initial_state;
    while let Some(event) = event_rx.recv().await {
        log::debug!("Applying event: {:?}", event);
        let event_applied = apply_event(&mut state, event);
        let (render_state, next_action) = match event_applied {
            EventApplied::Rejected => {
                log::warn!("Event rejected");
                (false, None)
            }
            EventApplied::Accepted { next_action } => (next_action.is_none(), next_action),
            EventApplied::StateChanged { next_action } => (true, next_action),
        };
        if render_state {
            log::debug!("Rendering current state: {:?}", state);
            render_state_fn(&state, &event_tx);
        }
        if let Some(next_action) = next_action {
            if matches!(next_action, Action::Terminate) {
                break;
            }
            let shared_env = shared_env.clone();
            let event_tx = event_tx.clone();
            log::debug!("Dispatching next action: {:?}", next_action);
            tokio::spawn(dispatch_action(shared_env, event_tx, next_action));
        }
    }
}

fn apply_event(state: &mut State, event: Event) -> EventApplied<Action> {
    match event {
        Event::Effect(Effect::ErrorOccurred(error))
        | Event::CollectionEvent(collection::Event::Effect(collection::Effect::ErrorOccurred(
            error,
        )))
        | Event::MediaTrackerEvent(media::tracker::Event::Effect(
            media::tracker::Effect::ErrorOccurred(error),
        )) => {
            state.errors.push(error);
            EventApplied::StateChanged { next_action: None }
        }
        Event::CollectionEvent(event) => {
            event_applied(collection::apply_event(&mut state.collection, event))
        }
        Event::MediaTrackerEvent(event) => {
            event_applied(media::tracker::apply_event(&mut state.media_tracker, event))
        }
        Event::Intent(intent) => match intent {
            Intent::EmitDeferredEvent {
                emit_not_before,
                event,
            } => EventApplied::Accepted {
                next_action: Some(Action::EmitDeferredEvent {
                    emit_not_before,
                    event,
                }),
            },
            Intent::RenderState => EventApplied::StateChanged { next_action: None },
            Intent::Terminate => {
                if !state.media_tracker.is_idle() {
                    return EventApplied::Rejected;
                }
                EventApplied::Accepted {
                    next_action: Some(Action::Terminate),
                }
            }
        },
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
            send_event(&event_tx, *event);
        }
        Action::Terminate => unreachable!(),
    }
}

async fn receive_response_body(response: Response) -> anyhow::Result<Bytes> {
    let response_status = response.status();
    let bytes = response.bytes().await?;
    if !response_status.is_success() {
        let json = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap_or_default();
        let err = if json.is_null() {
            anyhow::anyhow!("{}", response_status)
        } else {
            anyhow::anyhow!("{} {}", response_status, json)
        };
        return Err(err);
    }
    Ok(bytes)
}
