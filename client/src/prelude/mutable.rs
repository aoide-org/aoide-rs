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

use super::{
    send_message, Action, AsyncTask, Environment, Message, MessageChannel, MessageLoopControl,
    MessageLoopState, MessageSender, RenderModelFn,
};

use std::{
    fmt,
    ops::{Add, AddAssign},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ModelMutation {
    Unchanged,
    MaybeChanged,
}

impl Add<ModelMutation> for ModelMutation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self == Self::Unchanged && rhs == Self::Unchanged {
            Self::Unchanged
        } else {
            Self::MaybeChanged
        }
    }
}

impl AddAssign for ModelMutation {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct ModelUpdated<E, T> {
    pub state_mutation: ModelMutation,
    pub next_action: Option<Action<E, T>>,
}

impl<E, T> ModelUpdated<E, T> {
    pub fn unchanged(next_action: impl Into<Option<Action<E, T>>>) -> Self {
        Self {
            state_mutation: ModelMutation::Unchanged,
            next_action: next_action.into(),
        }
    }

    pub fn maybe_changed(next_action: impl Into<Option<Action<E, T>>>) -> Self {
        Self {
            state_mutation: ModelMutation::MaybeChanged,
            next_action: next_action.into(),
        }
    }
}

pub fn model_updated<E1, E2, T1, T2>(from: ModelUpdated<E1, T1>) -> ModelUpdated<E2, T2>
where
    E1: Into<E2>,
    T1: Into<T2>,
{
    let ModelUpdated {
        state_mutation,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    ModelUpdated {
        state_mutation,
        next_action,
    }
}

pub trait Model {
    type Intent;
    type Effect;
    type Task;

    fn update(
        &mut self,
        message: Message<Self::Intent, Self::Effect>,
    ) -> ModelUpdated<Self::Effect, Self::Task>;
}

pub(crate) fn handle_next_message<M>(
    state: MessageLoopState,
    shared_env: &Arc<Environment>,
    model: &mut M,
    message_tx: &MessageSender<M::Intent, M::Effect>,
    mut next_message: Message<M::Intent, M::Effect>,
    render_fn: &mut RenderModelFn<M, M::Intent>,
) -> MessageLoopControl
where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug + Send + 'static,
    M::Effect: fmt::Debug + Send + 'static,
    M::Task: AsyncTask<M::Effect> + fmt::Debug + 'static,
{
    let mut state_mutation = ModelMutation::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let ModelUpdated {
            state_mutation: next_state_mutation,
            next_action,
        } = model.update(next_message);
        state_mutation += next_state_mutation;
        if let Some(next_action) = next_action {
            number_of_next_actions += 1;
            match next_action {
                Action::ApplyEffect(effect) => {
                    log::debug!("Applying subsequent effect immediately: {:?}", effect);
                    next_message = Message::Effect(effect);
                    continue 'process_next_message;
                }
                Action::DispatchTask(task) => match state {
                    MessageLoopState::Running => {
                        log::debug!("Dispatching task asynchronously: {:?}", task);
                        Environment::dispatch_task(
                            shared_env.clone(),
                            message_tx.clone(),
                            task.execute(shared_env.clone()),
                        );
                        number_of_tasks_dispatched += 1;
                    }
                    MessageLoopState::Terminating => {
                        log::warn!(
                            "Cannot dispatch new asynchronous task while terminating: {:?}",
                            task
                        );
                    }
                },
            }
        }
        if state_mutation == ModelMutation::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current state: {:?}", model);
            if let Some(rendering_intent) = render_fn(&model) {
                log::debug!(
                    "Received intent after rendering state: {:?}",
                    rendering_intent
                );
                send_message(&message_tx, Message::Intent(rendering_intent));
                number_of_messages_sent += 1;
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

pub async fn message_loop<M>(
    shared_env: Arc<Environment>,
    (message_tx, mut message_rx): MessageChannel<M::Intent, M::Effect>,
    mut model: M,
    mut render_model_fn: Box<RenderModelFn<M, M::Intent>>,
) -> M
where
    M: Model + fmt::Debug,
    M::Intent: fmt::Debug + Send + 'static,
    M::Effect: fmt::Debug + Send + 'static,
    M::Task: AsyncTask<M::Effect> + fmt::Debug + 'static,
{
    let mut state = MessageLoopState::Running;
    while let Some(next_message) = message_rx.recv().await {
        match handle_next_message(
            state,
            &shared_env,
            &mut model,
            &message_tx,
            next_message,
            &mut *render_model_fn,
        ) {
            MessageLoopControl::Continue => match state {
                MessageLoopState::Running => (),
                MessageLoopState::Terminating => {
                    if shared_env.all_tasks_finished() {
                        break;
                    }
                    log::debug!("Continuing message loop until all pending tasks have finished");
                }
            },
            MessageLoopControl::Terminate => {
                state = MessageLoopState::Terminating;
                if shared_env.all_tasks_finished() {
                    break;
                }
                log::debug!("Continuing message loop until all pending tasks have finished");
            }
        }
    }
    log::debug!("Terminated message loop");
    model
}
