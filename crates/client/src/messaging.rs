// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{fmt, sync::Arc};

use futures::{channel::mpsc, StreamExt as _};

use crate::{
    action::Action,
    message::Message,
    state::{RenderStateFn, State, StateChanged, StateUpdated},
};

pub type MessageSender<Intent, Effect> = mpsc::Sender<Message<Intent, Effect>>;
pub type MessageReceiver<Intent, Effect> = mpsc::Receiver<Message<Intent, Effect>>;
pub type MessageChannel<Intent, Effect> = (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
);

/// Create a buffered message channel with limited capacity.
#[must_use]
pub fn message_channel<Intent, Effect>(
    capacity: usize,
) -> (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
) {
    mpsc::channel(capacity)
}

pub fn send_message<Intent: fmt::Debug, Effect: fmt::Debug>(
    message_tx: &mut MessageSender<Intent, Effect>,
    message: impl Into<Message<Intent, Effect>>,
) {
    let message = message.into();
    log::debug!("Sending message: {message:?}");
    if let Err(err) = message_tx.try_send(message) {
        // Channel is closed, i.e. receiver has been dropped
        log::debug!("Failed to send message: {err}");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandled {
    Progressing,
    NoProgress,
}

pub fn handle_next_message<E, S>(
    shared_env: &Arc<E>,
    state: &mut S,
    message_tx: &mut MessageSender<S::Intent, S::Effect>,
    mut next_message: Message<S::Intent, S::Effect>,
    render_fn: &mut RenderStateFn<S, S::Intent>,
) -> MessageHandled
where
    E: TaskDispatcher<S::Intent, S::Effect, S::Task>,
    S: State + fmt::Debug,
    S::Intent: fmt::Debug + Send + 'static,
    S::Effect: fmt::Debug + Send + 'static,
    S::Task: fmt::Debug + 'static,
{
    let mut state_changed = StateChanged::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let StateUpdated {
            changed: next_state_changed,
            next_action,
        } = state.update(next_message);
        state_changed += next_state_changed;
        if let Some(next_action) = next_action {
            number_of_next_actions += 1;
            match next_action {
                Action::ApplyEffect(effect) => {
                    log::debug!("Applying subsequent effect immediately: {effect:?}");
                    next_message = Message::Effect(effect);
                    continue 'process_next_message;
                }
                Action::DispatchTask(task) => {
                    log::debug!("Dispatching task asynchronously: {task:?}");
                    shared_env.dispatch_task(shared_env.clone(), message_tx.clone(), task);
                    number_of_tasks_dispatched += 1;
                }
            }
        }
        if state_changed == StateChanged::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current state: {state:?}");
            if let Some(observation_intent) = render_fn(state) {
                log::debug!("Received intent after observing state: {observation_intent:?}");
                send_message(message_tx, Message::Intent(observation_intent));
                number_of_messages_sent += 1;
            }
        }
        break;
    }
    log::debug!("number_of_next_actions = {number_of_next_actions}, number_of_messages_sent = {number_of_messages_sent}, number_of_tasks_dispatched = {number_of_tasks_dispatched}");
    if number_of_messages_sent + number_of_tasks_dispatched > 0 {
        MessageHandled::Progressing
    } else {
        MessageHandled::NoProgress
    }
}

pub async fn message_loop<E, S>(
    shared_env: Arc<E>,
    (mut message_tx, mut message_rx): MessageChannel<S::Intent, S::Effect>,
    mut state: S,
    mut render_state_fn: Box<RenderStateFn<S, S::Intent>>,
) -> S
where
    E: TaskDispatcher<S::Intent, S::Effect, S::Task>,
    S: State + fmt::Debug,
    S::Intent: fmt::Debug + Send + 'static,
    S::Effect: fmt::Debug + Send + 'static,
    S::Task: fmt::Debug + 'static,
{
    while let Some(next_message) = message_rx.next().await {
        match handle_next_message(
            &shared_env,
            &mut state,
            &mut message_tx,
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

pub trait TaskDispatcher<Intent, Effect, Task> {
    fn all_tasks_finished(&self) -> bool;

    fn dispatch_task(
        &self,
        shared_self: Arc<Self>,
        message_tx: MessageSender<Intent, Effect>,
        task: Task,
    );
}
