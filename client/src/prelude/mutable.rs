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

use crate::prelude::MessageHandled;

use super::{
    send_message, Action, Message, MessageChannel, MessageSender, RenderStateFn,
    TaskDispatchEnvironment,
};

use std::{
    fmt,
    ops::{Add, AddAssign},
    sync::Arc,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StateMutation {
    Unchanged,
    MaybeChanged,
}

impl Add<StateMutation> for StateMutation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self == Self::Unchanged && rhs == Self::Unchanged {
            Self::Unchanged
        } else {
            Self::MaybeChanged
        }
    }
}

impl AddAssign for StateMutation {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StateUpdated<Effect, Task> {
    pub state_mutation: StateMutation,
    pub next_action: Option<Action<Effect, Task>>,
}

impl<Effect, Task> StateUpdated<Effect, Task> {
    pub fn unchanged(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            state_mutation: StateMutation::Unchanged,
            next_action: next_action.into(),
        }
    }

    pub fn maybe_changed(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            state_mutation: StateMutation::MaybeChanged,
            next_action: next_action.into(),
        }
    }
}

pub fn state_updated<E1, E2, T1, T2>(from: StateUpdated<E1, T1>) -> StateUpdated<E2, T2>
where
    E1: Into<E2>,
    T1: Into<T2>,
{
    let StateUpdated {
        state_mutation,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    StateUpdated {
        state_mutation,
        next_action,
    }
}

pub trait State {
    type Intent;
    type Effect;
    type Task;

    fn update(
        &mut self,
        message: Message<Self::Intent, Self::Effect>,
    ) -> StateUpdated<Self::Effect, Self::Task>;
}

pub fn handle_next_message<E, S>(
    shared_env: &Arc<E>,
    state: &mut S,
    message_tx: &MessageSender<S::Intent, S::Effect>,
    mut next_message: Message<S::Intent, S::Effect>,
    render_fn: &mut RenderStateFn<S, S::Intent>,
) -> MessageHandled
where
    E: TaskDispatchEnvironment<S::Intent, S::Effect, S::Task>,
    S: State + fmt::Debug,
    S::Intent: fmt::Debug + Send + 'static,
    S::Effect: fmt::Debug + Send + 'static,
    S::Task: fmt::Debug + 'static,
{
    let mut state_mutation = StateMutation::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let StateUpdated {
            state_mutation: next_state_mutation,
            next_action,
        } = state.update(next_message);
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
                    log::debug!("Dispatching task asynchronously: {:?}", task);
                    shared_env.dispatch_task(shared_env.clone(), message_tx.clone(), task);
                    number_of_tasks_dispatched += 1;
                }
            }
        }
        if state_mutation == StateMutation::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current state: {:?}", state);
            if let Some(observation_intent) = render_fn(state) {
                log::debug!(
                    "Received intent after observing state: {:?}",
                    observation_intent
                );
                send_message(message_tx, Message::Intent(observation_intent));
                number_of_messages_sent += 1;
            }
        }
        break;
    }
    log::debug!("number_of_next_actions = {}, number_of_messages_sent = {}, number_of_tasks_dispatched = {}", number_of_next_actions, number_of_messages_sent, number_of_tasks_dispatched);
    if number_of_messages_sent + number_of_tasks_dispatched > 0 {
        MessageHandled::Progressing
    } else {
        MessageHandled::NoProgress
    }
}

pub async fn message_loop<E, S>(
    shared_env: Arc<E>,
    (message_tx, mut message_rx): MessageChannel<S::Intent, S::Effect>,
    mut state: S,
    mut render_state_fn: Box<RenderStateFn<S, S::Intent>>,
) -> S
where
    E: TaskDispatchEnvironment<S::Intent, S::Effect, S::Task>,
    S: State + fmt::Debug,
    S::Intent: fmt::Debug + Send + 'static,
    S::Effect: fmt::Debug + Send + 'static,
    S::Task: fmt::Debug + 'static,
{
    while let Some(next_message) = message_rx.recv().await {
        match handle_next_message(
            &shared_env,
            &mut state,
            &message_tx,
            next_message,
            &mut *render_state_fn,
        ) {
            MessageHandled::Progressing => (),
            MessageHandled::NoProgress => {
                if shared_env.all_tasks_finished() {
                    break;
                }
            }
        }
    }
    log::debug!("Terminated message loop");
    state
}
