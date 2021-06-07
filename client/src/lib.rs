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

pub mod prelude;

pub mod collection;
pub mod media_tracker;
pub mod track_search;

use crate::prelude::{
    mutable::{state_updated, State as MutableState},
    send_message,
};

use bytes::Bytes;
use reqwest::{Client, Response, Url};
use std::{
    num::NonZeroUsize,
    sync::{atomic::AtomicUsize, Arc},
    time::Instant,
};

pub type Action = crate::prelude::Action<Effect, Task>;
pub type Message = crate::prelude::Message<Intent, Effect>;
pub type MessageSender = crate::prelude::MessageSender<Intent, Effect>;
pub type MessageReceiver = crate::prelude::MessageReceiver<Intent, Effect>;
pub type MessageChannel = crate::prelude::MessageChannel<Intent, Effect>;
pub type StateUpdated = crate::prelude::mutable::StateUpdated<Effect, Task>;

#[derive(Debug, Default)]
pub struct State {
    last_errors: Vec<anyhow::Error>,
    terminating: bool,
    pub collection: collection::State,
    pub media_tracker: media_tracker::State,
}

impl State {
    pub fn last_errors(&self) -> &[anyhow::Error] {
        &self.last_errors
    }
}

impl MutableState for State {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn update(&mut self, message: Message) -> StateUpdated {
        log::debug!("Updating state {:?} with message {:?}", self, message);
        match message {
            Message::Intent(intent) => intent.apply_on(self),
            Message::Effect(effect) => effect.apply_on(self),
        }
    }
}

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
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    InjectEffect(Box<Effect>),
    Terminate,
    DiscardFirstErrors(NonZeroUsize),
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
    FirstErrorsDiscarded(NonZeroUsize),
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

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::debug!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::RenderState => StateUpdated::maybe_changed(None),
            Self::TimedIntent { not_before, intent } => {
                if state.terminating {
                    log::debug!("Discarding timed intent while terminating: {:?}", intent);
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::unchanged(Action::dispatch_task(Task::TimedIntent {
                    not_before,
                    intent,
                }))
            }
            Self::InjectEffect(effect) => StateUpdated::unchanged(Action::apply_effect(*effect)),
            Self::Terminate => state_updated(
                media_tracker::Intent::AbortOnTermination.apply_on(&mut state.media_tracker),
            ),
            Self::DiscardFirstErrors(num_errors_requested) => {
                let num_errors =
                    NonZeroUsize::new(num_errors_requested.get().min(state.last_errors.len()));
                let next_action = if let Some(num_errors) = num_errors {
                    if num_errors < num_errors_requested {
                        debug_assert!(num_errors_requested.get() > 1);
                        log::debug!(
                            "Discarding only {} instead of {} errors",
                            num_errors,
                            num_errors_requested
                        );
                    }
                    Some(Action::apply_effect(Effect::FirstErrorsDiscarded(
                        num_errors,
                    )))
                } else {
                    log::debug!("No errors to discard");
                    None
                };
                StateUpdated::unchanged(next_action)
            }
            Self::CollectionIntent(intent) => state_updated(intent.apply_on(&mut state.collection)),
            Self::MediaTrackerIntent(intent) => {
                state_updated(intent.apply_on(&mut state.media_tracker))
            }
        }
    }
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::debug!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ErrorOccurred(error)
            | Self::CollectionEffect(collection::Effect::ErrorOccurred(error))
            | Self::MediaTrackerEffect(media_tracker::Effect::ErrorOccurred(error)) => {
                state.last_errors.push(error);
                StateUpdated::maybe_changed(None)
            }
            Self::FirstErrorsDiscarded(num_errors) => {
                debug_assert!(num_errors.get() <= state.last_errors.len());
                state.last_errors = state.last_errors.drain(num_errors.get()..).collect();
                StateUpdated::maybe_changed(None)
            }
            Self::ApplyIntent(intent) => intent.apply_on(state),
            Self::CollectionEffect(effect) => state_updated(effect.apply_on(&mut state.collection)),
            Self::MediaTrackerEffect(effect) => {
                state_updated(effect.apply_on(&mut state.media_tracker))
            }
        }
    }
}

/// Immutable environment
#[derive(Debug)]
pub struct Environment {
    api_url: Url,
    client: Client,
    pending_tasks_count: AtomicUsize,
}

impl Environment {
    pub fn new(api_url: Url) -> Self {
        Self {
            api_url,
            client: Client::new(),
            pending_tasks_count: AtomicUsize::new(0),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn join_api_url(&self, input: &str) -> anyhow::Result<Url> {
        self.api_url.join(input).map_err(Into::into)
    }
}

impl crate::prelude::Environment<Intent, Effect, Task> for Environment {
    fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_count
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }

    fn dispatch_task(&self, shared_self: Arc<Self>, message_tx: MessageSender, task: Task) {
        shared_self
            .pending_tasks_count
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        tokio::spawn(async move {
            log::debug!("Executing task: {:?}", task);
            let effect = match task {
                Task::TimedIntent { not_before, intent } => {
                    tokio::time::sleep_until(not_before.into()).await;
                    Effect::ApplyIntent(*intent)
                }
                Task::Collection(task) => task.execute_with(&shared_self).await.into(),
                Task::MediaTracker(task) => task.execute_with(&shared_self).await.into(),
            };
            log::debug!("Received effect from task: {:?}", effect);
            send_message(&message_tx, Message::Effect(effect));
            shared_self
                .pending_tasks_count
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        });
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
