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
use tokio::signal;

use crate::media::tracker::abort;

#[derive(Debug, Default)]
pub struct State {
    last_errors: Vec<anyhow::Error>,
    pub collection: collection::State,
    pub media_tracker: media::tracker::State,
}

impl State {
    pub fn last_errors(&self) -> &[anyhow::Error] {
        &self.last_errors
    }
}

#[derive(Debug)]
pub enum NextAction {
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    Collection(collection::NextAction),
    MediaTracker(media::tracker::NextAction),
}

impl From<collection::NextAction> for NextAction {
    fn from(next_action: collection::NextAction) -> Self {
        Self::Collection(next_action)
    }
}

impl From<media::tracker::NextAction> for NextAction {
    fn from(next_action: media::tracker::NextAction) -> Self {
        Self::MediaTracker(next_action)
    }
}

#[derive(Debug)]
pub enum Event {
    Intent(Intent),
    Effect(Effect),
}

impl From<Intent> for Event {
    fn from(intent: Intent) -> Self {
        Self::Intent(intent)
    }
}

impl From<Effect> for Event {
    fn from(effect: Effect) -> Self {
        Self::Effect(effect)
    }
}

impl From<collection::Intent> for Event {
    fn from(intent: collection::Intent) -> Self {
        Self::Intent(intent.into())
    }
}

impl From<collection::Effect> for Event {
    fn from(effect: collection::Effect) -> Self {
        Self::Effect(effect.into())
    }
}

impl From<media::tracker::Intent> for Event {
    fn from(intent: media::tracker::Intent) -> Self {
        Self::Intent(intent.into())
    }
}

impl From<media::tracker::Effect> for Event {
    fn from(effect: media::tracker::Effect) -> Self {
        Self::Effect(effect.into())
    }
}

#[derive(Debug)]
pub enum Intent {
    RenderState,
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    CollectionIntent(collection::Intent),
    MediaTrackerIntent(media::tracker::Intent),
}

impl From<collection::Intent> for Intent {
    fn from(intent: collection::Intent) -> Self {
        Self::CollectionIntent(intent)
    }
}

impl From<media::tracker::Intent> for Intent {
    fn from(intent: media::tracker::Intent) -> Self {
        Self::MediaTrackerIntent(intent)
    }
}

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
    CollectionEffect(collection::Effect),
    MediaTrackerEffect(media::tracker::Effect),
}

impl From<collection::Effect> for Effect {
    fn from(effect: collection::Effect) -> Self {
        Self::CollectionEffect(effect)
    }
}

impl From<media::tracker::Effect> for Effect {
    fn from(effect: media::tracker::Effect) -> Self {
        Self::MediaTrackerEffect(effect)
    }
}

pub type RenderStateFn = dyn FnMut(&State) -> Option<Intent> + Send;

pub async fn handle_events(
    env: Environment,
    event_channel: (EventSender<Event>, EventReceiver<Event>),
    initial_state: State,
    mut render_state_fn: Box<RenderStateFn>,
) {
    let shared_env = Arc::new(env);
    let (event_tx, mut event_rx) = event_channel;
    let mut state = initial_state;
    let mut state_rendered_after_last_event = false;
    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                let (state_mutation, next_action) =
                apply_event(&mut state, state_rendered_after_last_event, event);
                state_rendered_after_last_event = false;
                let mut terminate = true;
                if state_mutation == StateMutation::MaybeChanged || next_action.is_none() {
                    log::debug!("Rendering current state: {:?}", state);
                    if let Some(rendering_intent) = render_state_fn(&state) {
                        log::debug!(
                            "Received intent after rendering state: {:?}",
                            rendering_intent
                        );
                        emit_event(&event_tx, rendering_intent);
                        terminate = false;
                    }
                    state_rendered_after_last_event = true;
                }
                if let Some(next_action) = next_action {
                    let shared_env = shared_env.clone();
                    let event_tx = event_tx.clone();
                    log::debug!("Dispatching next action asynchronously: {:?}", next_action);
                    tokio::spawn(async move {
                        if let Some(event) = dispatch_next_action(shared_env, next_action).await {
                            log::debug!("Received event after dispatching action: {:?}", event);
                            emit_event(&event_tx, event);
                        }
                    });
                    terminate = false;
                }
                if terminate {
                    break;
                }
            }
            _ = signal::ctrl_c() => {
                log::info!("Aborting after receiving SIGINT...");
                emit_event(&event_tx, abort());
            }
            else => {
                // Exit the message loop in all other cases, i.e. if event_rx.recv()
                // returned None after the channel has been closed
                break;
            }
        }
    }
}

fn apply_event(
    state: &mut State,
    state_rendered_after_last_event: bool,
    event: Event,
) -> (StateMutation, Option<NextAction>) {
    log::debug!("Applying event: {:?}", event);
    if state_rendered_after_last_event {
        // Consume errors only once, i.e. clear after rendering the state
        state.last_errors.clear();
    }
    match event {
        Event::Intent(intent) => apply_intent(state, intent),
        Event::Effect(effect) => apply_effect(state, effect),
    }
}

fn apply_intent(state: &mut State, intent: Intent) -> (StateMutation, Option<NextAction>) {
    match intent {
        Intent::RenderState => (StateMutation::MaybeChanged, None),
        Intent::TimedIntent { not_before, intent } => (
            StateMutation::Unchanged,
            Some(NextAction::TimedIntent { not_before, intent }),
        ),
        Intent::CollectionIntent(intent) => {
            event_applied(collection::apply_intent(&mut state.collection, intent))
        }
        Intent::MediaTrackerIntent(intent) => event_applied(media::tracker::apply_intent(
            &mut state.media_tracker,
            intent,
        )),
    }
}

fn apply_effect(state: &mut State, effect: Effect) -> (StateMutation, Option<NextAction>) {
    match effect {
        Effect::ErrorOccurred(error)
        | Effect::CollectionEffect(collection::Effect::ErrorOccurred(error))
        | Effect::MediaTrackerEffect(media::tracker::Effect::ErrorOccurred(error)) => {
            state.last_errors.push(error);
            (StateMutation::MaybeChanged, None)
        }
        Effect::CollectionEffect(effect) => {
            event_applied(collection::apply_effect(&mut state.collection, effect))
        }
        Effect::MediaTrackerEffect(effect) => event_applied(media::tracker::apply_effect(
            &mut state.media_tracker,
            effect,
        )),
    }
}

async fn dispatch_next_action(
    shared_env: Arc<Environment>,
    next_action: NextAction,
) -> Option<Event> {
    log::debug!("Dispatching next action: {:?}", next_action);
    match next_action {
        NextAction::TimedIntent { not_before, intent } => {
            tokio::time::sleep_until(not_before.into()).await;
            let unboxed_intent = *intent;
            // In this special case the action results in an intent and not an effect!
            // TODO: How could this be resolved while preserving the partitioning into
            // intents and effects?
            Some(unboxed_intent.into())
        }
        NextAction::Collection(next_action) => {
            collection::dispatch_next_action(shared_env, next_action)
                .await
                .map(Into::into)
        }
        NextAction::MediaTracker(action) => {
            media::tracker::dispatch_next_action(shared_env, action)
                .await
                .map(Into::into)
        }
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
