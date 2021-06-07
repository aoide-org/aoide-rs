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

use crate::prelude::mutable::model_updated;

use self::{media::tracker as media_tracker, prelude::*};

use async_trait::async_trait;
use bytes::Bytes;
use prelude::mutable::Model;
use reqwest::Response;
use std::{sync::Arc, time::Instant};

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

pub type Message = crate::prelude::Message<Intent, Effect>;
pub type Action = crate::prelude::Action<Effect, Task>;
pub type ModelUpdated = crate::prelude::mutable::ModelUpdated<Effect, Task>;

impl Model for State {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn update(&mut self, message: Message) -> ModelUpdated {
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
    InjectEffect(Box<Effect>),
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    Terminate,
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

impl Intent {
    pub fn apply_on(self, state: &mut State) -> ModelUpdated {
        log::debug!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::RenderState => ModelUpdated::maybe_changed(None),
            Self::InjectEffect(effect) => ModelUpdated::unchanged(Action::apply_effect(*effect)),
            Self::TimedIntent { not_before, intent } => {
                ModelUpdated::unchanged(Action::dispatch_task(Task::TimedIntent {
                    not_before,
                    intent,
                }))
            }
            Self::Terminate => model_updated(
                media_tracker::Intent::AbortOnTermination.apply_on(&mut state.media_tracker),
            ),
            Self::ClearFirstErrorsBeforeNextRenderState(head_len) => {
                ModelUpdated::unchanged(Action::apply_effect(Effect::ClearFirstErrors(head_len)))
            }
            Self::CollectionIntent(intent) => model_updated(intent.apply_on(&mut state.collection)),
            Self::MediaTrackerIntent(intent) => {
                model_updated(intent.apply_on(&mut state.media_tracker))
            }
        }
    }
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> ModelUpdated {
        log::debug!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ErrorOccurred(error)
            | Self::CollectionEffect(collection::Effect::ErrorOccurred(error))
            | Self::MediaTrackerEffect(media_tracker::Effect::ErrorOccurred(error)) => {
                state.last_errors.push(error);
                ModelUpdated::maybe_changed(None)
            }
            Self::ClearFirstErrors(head_len) => {
                debug_assert!(head_len <= state.last_errors.len());
                state.last_errors = state.last_errors.drain(head_len..).collect();
                ModelUpdated::maybe_changed(None)
            }
            Self::ApplyIntent(intent) => intent.apply_on(state),
            Self::CollectionEffect(effect) => model_updated(effect.apply_on(&mut state.collection)),
            Self::MediaTrackerEffect(effect) => {
                model_updated(effect.apply_on(&mut state.media_tracker))
            }
        }
    }
}

#[async_trait]
impl AsyncTask<Effect> for Task {
    async fn execute(self, shared_env: Arc<Environment>) -> Effect {
        log::debug!("Executing task: {:?}", self);
        match self {
            Self::TimedIntent { not_before, intent } => {
                tokio::time::sleep_until(not_before.into()).await;
                Effect::ApplyIntent(*intent)
            }
            Self::Collection(task) => task.execute_with(&shared_env).await.into(),
            Self::MediaTracker(task) => task.execute_with(&shared_env).await.into(),
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
