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

pub type Action = crate::prelude::Action<Effect, Task>;

impl From<collection::Effect> for Action {
    fn from(effect: collection::Effect) -> Self {
        Self::ApplyEffect(effect.into())
    }
}

impl From<collection::Task> for Action {
    fn from(task: collection::Task) -> Self {
        Self::DispatchTask(task.into())
    }
}

impl From<media::tracker::Effect> for Action {
    fn from(effect: media::tracker::Effect) -> Self {
        Self::ApplyEffect(effect.into())
    }
}

impl From<media::tracker::Task> for Action {
    fn from(task: media::tracker::Task) -> Self {
        Self::DispatchTask(task.into())
    }
}

#[derive(Debug)]
pub enum Task {
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    Collection(collection::Task),
    MediaTracker(media::tracker::Task),
}

impl From<collection::Task> for Task {
    fn from(task: collection::Task) -> Self {
        Self::Collection(task)
    }
}

impl From<collection::Action> for Action {
    fn from(action: collection::Action) -> Self {
        match action {
            collection::Action::ApplyEffect(effect) => effect.into(),
            collection::Action::DispatchTask(task) => task.into(),
        }
    }
}

impl From<media::tracker::Task> for Task {
    fn from(task: media::tracker::Task) -> Self {
        Self::MediaTracker(task)
    }
}

impl From<media::tracker::Action> for Action {
    fn from(action: media::tracker::Action) -> Self {
        match action {
            media::tracker::Action::ApplyEffect(effect) => effect.into(),
            media::tracker::Action::DispatchTask(task) => task.into(),
        }
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
    ClearFirstErrorsBeforeNextRenderState(usize),
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
    ClearFirstErrors(usize),
    ApplyIntent(Intent),
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
    initial_state: State,
    initial_intent: Intent,
    mut render_state_fn: Box<RenderStateFn>,
) {
    let shared_env = Arc::new(env);
    let mut state = initial_state;
    let (event_tx, mut event_rx) = event_channel();
    // Kick off the loop by emitting an initial event
    emit_event(&event_tx, initial_intent);
    let mut event_tx = Some(event_tx);
    let mut terminating = false;
    loop {
        tokio::select! {
            Some(next_event) = event_rx.recv() => {
                match handle_next_event(&shared_env, event_tx.as_ref(), &mut state, &mut *render_state_fn, next_event) {
                    EventLoopControl::Continue => (),
                    EventLoopControl::Terminate => {
                        if !terminating {
                            log::debug!("Terminating...");
                            terminating = true;
                        }
                    }
                }
                if terminating && event_tx.is_some() && shared_env.all_tasks_finished() {
                    log::debug!("Closing event emitter after all pending tasks finished");
                    event_tx = None;
                }
            }
            _ = signal::ctrl_c(), if !terminating => {
                log::info!("Aborting after receiving SIGINT...");
                debug_assert!(event_tx.is_some());
                emit_event(event_tx.as_ref().unwrap(), abort());
            }
            else => {
                // Exit the message loop in all other cases, i.e. if event_rx.recv()
                // returned None after the channel has been closed
                break;
            }
        }
    }
    debug_assert!(terminating);
    debug_assert!(event_tx.is_none());
}

#[derive(Debug)]
enum EventLoopControl {
    Continue,
    Terminate,
}

fn handle_next_event(
    shared_env: &Arc<Environment>,
    event_tx: Option<&EventSender<Event>>,
    state: &mut State,
    render_state_fn: &mut RenderStateFn,
    mut next_event: Event,
) -> EventLoopControl {
    let mut state_mutation = StateMutation::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_events_emitted = 0;
    let mut number_of_tasks_dispatched = 0;
    'apply_next_event: loop {
        let (next_state_mutation, next_action) = next_event.apply_on(state);
        state_mutation += next_state_mutation;
        if let Some(next_action) = next_action {
            number_of_next_actions += 1;
            match next_action {
                Action::ApplyEffect(effect) => {
                    log::debug!("Applying effect immediately: {:?}", effect);
                    next_event = Event::Effect(effect);
                    continue 'apply_next_event;
                }
                Action::DispatchTask(task) => {
                    if let Some(event_tx) = event_tx {
                        let shared_env = shared_env.clone();
                        let event_tx = event_tx.clone();
                        log::debug!("Dispatching task asynchronously: {:?}", task);
                        shared_env.task_pending();
                        tokio::spawn(async move {
                            let effect = task.execute_with(&shared_env).await;
                            log::debug!("Received effect from task: {:?}", effect);
                            emit_event(&event_tx, effect);
                            shared_env.task_finished();
                        });
                        number_of_tasks_dispatched += 1;
                    } else {
                        log::warn!(
                            "Cannot dispatch new asynchronous task while terminating: {:?}",
                            task
                        );
                    }
                }
            }
        }
        if state_mutation == StateMutation::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current state: {:?}", state);
            if let Some(rendering_intent) = render_state_fn(&state) {
                if let Some(event_tx) = event_tx {
                    log::debug!(
                        "Received intent after rendering state: {:?}",
                        rendering_intent
                    );
                    emit_event(&event_tx, rendering_intent);
                    number_of_events_emitted += 1;
                } else {
                    // Cannot emit any new events when draining the event channel
                    log::warn!(
                        "Dropping intent received after rendering state: {:?}",
                        rendering_intent
                    );
                }
            }
        }
        break;
    }
    log::debug!("number_of_next_actions = {}, number_of_events_emitted = {}, number_of_tasks_dispatched = {}", number_of_next_actions, number_of_events_emitted, number_of_tasks_dispatched);
    if number_of_events_emitted + number_of_tasks_dispatched > 0 {
        EventLoopControl::Continue
    } else {
        EventLoopControl::Terminate
    }
}

impl Event {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::debug!("Applying event {:?} on {:?}", self, state);
        match self {
            Self::Intent(intent) => intent.apply_on(state),
            Self::Effect(effect) => effect.apply_on(state),
        }
    }
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::debug!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::RenderState => (StateMutation::MaybeChanged, None),
            Self::ClearFirstErrorsBeforeNextRenderState(head_len) => (
                StateMutation::Unchanged,
                Some(Action::apply_effect(Effect::ClearFirstErrors(head_len))),
            ),
            Self::TimedIntent { not_before, intent } => (
                StateMutation::Unchanged,
                Some(Action::dispatch_task(Task::TimedIntent {
                    not_before,
                    intent,
                })),
            ),
            Self::CollectionIntent(intent) => event_applied(intent.apply_on(&mut state.collection)),
            Self::MediaTrackerIntent(intent) => {
                event_applied(intent.apply_on(&mut state.media_tracker))
            }
        }
    }
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::debug!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ErrorOccurred(error)
            | Self::CollectionEffect(collection::Effect::ErrorOccurred(error))
            | Self::MediaTrackerEffect(media::tracker::Effect::ErrorOccurred(error)) => {
                state.last_errors.push(error);
                (StateMutation::MaybeChanged, None)
            }
            Self::ClearFirstErrors(head_len) => {
                debug_assert!(head_len <= state.last_errors.len());
                state.last_errors = state.last_errors.drain(head_len..).collect();
                (StateMutation::MaybeChanged, None)
            }
            Self::ApplyIntent(intent) => intent.apply_on(state),
            Self::CollectionEffect(effect) => event_applied(effect.apply_on(&mut state.collection)),
            Self::MediaTrackerEffect(effect) => {
                event_applied(effect.apply_on(&mut state.media_tracker))
            }
        }
    }
}

impl Task {
    pub async fn execute_with(self, env: &Environment) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::TimedIntent { not_before, intent } => {
                tokio::time::sleep_until(not_before.into()).await;
                let unboxed_intent = *intent;
                Effect::ApplyIntent(unboxed_intent.into())
            }
            Self::Collection(task) => task.execute_with(env).await.into(),
            Self::MediaTracker(task) => task.execute_with(env).await.into(),
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
