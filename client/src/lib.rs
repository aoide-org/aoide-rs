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

use self::{media::tracker as media_tracker, prelude::*};

use bytes::Bytes;
use reqwest::Response;
use std::{sync::Arc, time::Instant};
use tokio::signal;

#[derive(Debug, Default)]
pub struct State {
    last_errors: Vec<anyhow::Error>,
    pub collection: collection::State,
    pub media_tracker: media_tracker::State,
}

impl State {
    pub fn last_errors(&self) -> &[anyhow::Error] {
        &self.last_errors
    }
}

impl State {
    pub fn update_from_message(&mut self, message: Message) -> (StateMutation, Option<Action>) {
        log::debug!("Processing message {:?} in state {:?}", message, self);
        match message {
            Message::Intent(intent) => intent.apply_on(self),
            Message::Effect(effect) => effect.apply_on(self),
        }
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

impl From<media_tracker::Effect> for Action {
    fn from(effect: media_tracker::Effect) -> Self {
        Self::ApplyEffect(effect.into())
    }
}

impl From<media_tracker::Task> for Action {
    fn from(task: media_tracker::Task) -> Self {
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
    MediaTracker(media_tracker::Task),
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

impl From<media_tracker::Task> for Task {
    fn from(task: media_tracker::Task) -> Self {
        Self::MediaTracker(task)
    }
}

impl From<media_tracker::Action> for Action {
    fn from(action: media_tracker::Action) -> Self {
        match action {
            media_tracker::Action::ApplyEffect(effect) => effect.into(),
            media_tracker::Action::DispatchTask(task) => task.into(),
        }
    }
}

#[derive(Debug)]
pub enum Message {
    Intent(Intent),
    Effect(Effect),
}

impl From<Intent> for Message {
    fn from(intent: Intent) -> Self {
        Self::Intent(intent)
    }
}

impl From<Effect> for Message {
    fn from(effect: Effect) -> Self {
        Self::Effect(effect)
    }
}

impl From<collection::Intent> for Message {
    fn from(intent: collection::Intent) -> Self {
        Self::Intent(intent.into())
    }
}

impl From<collection::Effect> for Message {
    fn from(effect: collection::Effect) -> Self {
        Self::Effect(effect.into())
    }
}

impl From<media_tracker::Intent> for Message {
    fn from(intent: media_tracker::Intent) -> Self {
        Self::Intent(intent.into())
    }
}

impl From<media_tracker::Effect> for Message {
    fn from(effect: media_tracker::Effect) -> Self {
        Self::Effect(effect.into())
    }
}

#[derive(Debug)]
pub enum Intent {
    RenderState,
    InjectEffect(Box<Effect>),
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    ClearFirstErrorsBeforeNextRenderState(usize),
    CollectionIntent(collection::Intent),
    MediaTrackerIntent(media_tracker::Intent),
}

impl From<collection::Intent> for Intent {
    fn from(intent: collection::Intent) -> Self {
        Self::CollectionIntent(intent)
    }
}

impl From<media_tracker::Intent> for Intent {
    fn from(intent: media_tracker::Intent) -> Self {
        Self::MediaTrackerIntent(intent)
    }
}

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
    ClearFirstErrors(usize),
    ApplyIntent(Intent),
    CollectionEffect(collection::Effect),
    MediaTrackerEffect(media_tracker::Effect),
}

impl From<collection::Effect> for Effect {
    fn from(effect: collection::Effect) -> Self {
        Self::CollectionEffect(effect)
    }
}

impl From<media_tracker::Effect> for Effect {
    fn from(effect: media_tracker::Effect) -> Self {
        Self::MediaTrackerEffect(effect)
    }
}

pub type RenderStateFn = dyn FnMut(&State) -> Option<Intent> + Send;

pub async fn handle_messages(
    shared_env: Arc<Environment>,
    initial_state: State,
    initial_intent: Intent,
    mut render_state_fn: Box<RenderStateFn>,
) -> State {
    let mut state = initial_state;
    let (message_tx, mut message_rx) = message_channel();
    // Kick off the loop by sending an initial message
    send_message(&message_tx, initial_intent);
    let mut message_tx = Some(message_tx);
    let mut terminating = false;
    loop {
        tokio::select! {
            Some(next_message) = message_rx.recv() => {
                match handle_next_message(&shared_env, message_tx.as_ref(), &mut state, &mut *render_state_fn, next_message) {
                    MessageLoopControl::Continue => (),
                    MessageLoopControl::Terminate => {
                        if !terminating {
                            log::debug!("Terminating...");
                            terminating = true;
                        }
                    }
                }
                if terminating && message_tx.is_some() && shared_env.all_tasks_finished() {
                    log::debug!("Closing message sender after all pending tasks finished");
                    message_tx = None;
                }
            }
            _ = signal::ctrl_c(), if !terminating => {
                log::info!("Aborting after receiving SIGINT...");
                debug_assert!(message_tx.is_some());
                send_message(message_tx.as_ref().unwrap(), media_tracker::Intent::Abort);
            }
            else => {
                // Exit the message loop in all other cases, i.e. if message_rx.recv()
                // returned None after the channel has been closed
                break;
            }
        }
    }
    debug_assert!(terminating);
    debug_assert!(message_tx.is_none());
    state
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageLoopControl {
    Continue,
    Terminate,
}

pub fn handle_next_message(
    shared_env: &Arc<Environment>,
    message_tx: Option<&MessageSender<Message>>,
    state: &mut State,
    render_state_fn: &mut RenderStateFn,
    mut next_message: Message,
) -> MessageLoopControl {
    let mut state_mutation = StateMutation::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let (next_state_mutation, next_action) = state.update_from_message(next_message);
        state_mutation += next_state_mutation;
        if let Some(next_action) = next_action {
            number_of_next_actions += 1;
            match next_action {
                Action::ApplyEffect(effect) => {
                    log::debug!("Applying subsequent effect immediately: {:?}", effect);
                    next_message = Message::Effect(effect);
                    continue 'process_next_message;
                }
                Action::DispatchTask(task) => {
                    if let Some(message_tx) = message_tx {
                        let shared_env = shared_env.clone();
                        let message_tx = message_tx.clone();
                        log::debug!("Dispatching task asynchronously: {:?}", task);
                        shared_env.task_pending();
                        tokio::spawn(async move {
                            let effect = task.execute_with(&shared_env).await;
                            log::debug!("Received effect from task: {:?}", effect);
                            send_message(&message_tx, effect);
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
                if let Some(message_tx) = message_tx {
                    log::debug!(
                        "Received intent after rendering state: {:?}",
                        rendering_intent
                    );
                    send_message(&message_tx, rendering_intent);
                    number_of_messages_sent += 1;
                } else {
                    // Cannot send any new messages when draining the message channel
                    log::warn!(
                        "Dropping intent received after rendering state: {:?}",
                        rendering_intent
                    );
                }
            }
        }
        break;
    }
    log::debug!("number_of_next_actions = {}, number_of_messages_sent = {}, number_of_tasks_dispatched = {}", number_of_next_actions, number_of_messages_sent, number_of_tasks_dispatched);
    if number_of_messages_sent + number_of_tasks_dispatched > 0 {
        MessageLoopControl::Continue
    } else {
        MessageLoopControl::Terminate
    }
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::debug!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::RenderState => (StateMutation::MaybeChanged, None),
            Self::InjectEffect(effect) => (
                StateMutation::Unchanged,
                Some(Action::apply_effect(*effect)),
            ),
            Self::TimedIntent { not_before, intent } => (
                StateMutation::Unchanged,
                Some(Action::dispatch_task(Task::TimedIntent {
                    not_before,
                    intent,
                })),
            ),
            Self::ClearFirstErrorsBeforeNextRenderState(head_len) => (
                StateMutation::Unchanged,
                Some(Action::apply_effect(Effect::ClearFirstErrors(head_len))),
            ),
            Self::CollectionIntent(intent) => {
                message_applied(intent.apply_on(&mut state.collection))
            }
            Self::MediaTrackerIntent(intent) => {
                message_applied(intent.apply_on(&mut state.media_tracker))
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
            | Self::MediaTrackerEffect(media_tracker::Effect::ErrorOccurred(error)) => {
                state.last_errors.push(error);
                (StateMutation::MaybeChanged, None)
            }
            Self::ClearFirstErrors(head_len) => {
                debug_assert!(head_len <= state.last_errors.len());
                state.last_errors = state.last_errors.drain(head_len..).collect();
                (StateMutation::MaybeChanged, None)
            }
            Self::ApplyIntent(intent) => intent.apply_on(state),
            Self::CollectionEffect(effect) => {
                message_applied(effect.apply_on(&mut state.collection))
            }
            Self::MediaTrackerEffect(effect) => {
                message_applied(effect.apply_on(&mut state.media_tracker))
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
                Effect::ApplyIntent(*intent)
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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
