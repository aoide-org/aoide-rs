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

pub mod mutable;
pub mod remote;

use std::{fmt, sync::Arc};

use tokio::sync::mpsc;

pub trait Environment<Intent, Effect, Task> {
    fn all_tasks_finished(&self) -> bool;
    fn dispatch_task(
        &self,
        shared_self: Arc<Self>,
        message_tx: MessageSender<Intent, Effect>,
        task: Task,
    );
}

pub type MessageSender<I, E> = mpsc::UnboundedSender<Message<I, E>>;
pub type MessageReceiver<I, E> = mpsc::UnboundedReceiver<Message<I, E>>;
pub type MessageChannel<I, E> = (MessageSender<I, E>, MessageReceiver<I, E>);

// TODO: Better use a bounded channel in production?
pub fn message_channel<I, E>() -> (MessageSender<I, E>, MessageReceiver<I, E>) {
    mpsc::unbounded_channel()
}

pub fn send_message<I: fmt::Debug, E: fmt::Debug>(
    message_tx: &MessageSender<I, E>,
    message: impl Into<Message<I, E>>,
) {
    let message = message.into();
    log::debug!("Sending message: {:?}", message);
    if let Err(message) = message_tx.send(message) {
        // Channel is closed, i.e. receiver has been dropped
        log::debug!("Failed to send message: {:?}", message.0);
    }
}

pub type RenderModelFn<M, I> = dyn FnMut(&M) -> Option<I> + Send;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<I, E> {
    Intent(I),
    Effect(E),
}

impl<I, E> Message<I, E> {
    pub fn intent(intent: impl Into<I>) -> Self {
        Self::Intent(intent.into())
    }

    pub fn effect(effect: impl Into<E>) -> Self {
        Self::Effect(effect.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<E, T> {
    ApplyEffect(E),
    DispatchTask(T),
}

impl<E, T> Action<E, T> {
    pub fn apply_effect(effect: impl Into<E>) -> Self {
        Self::ApplyEffect(effect.into())
    }

    pub fn dispatch_task(task: impl Into<T>) -> Self {
        Self::DispatchTask(task.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandled {
    Progressing,
    NoProgress,
}
