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

use std::{
    fmt,
    sync::{atomic::AtomicUsize, Arc},
};

use tokio::sync::mpsc;

/// Generic Environment trait
pub trait TaskDispatchEnvironment<Intent, Effect, Task> {
    fn all_tasks_finished(&self) -> bool;
    fn dispatch_task(
        &self,
        shared_self: Arc<Self>,
        message_tx: MessageSender<Intent, Effect>,
        task: Task,
    );
}

pub type MessageSender<Intent, Effect> = mpsc::UnboundedSender<Message<Intent, Effect>>;
pub type MessageReceiver<Intent, Effect> = mpsc::UnboundedReceiver<Message<Intent, Effect>>;
pub type MessageChannel<Intent, Effect> = (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
);

// TODO: Better use a bounded channel in production?
pub fn message_channel<Intent, Effect>() -> (
    MessageSender<Intent, Effect>,
    MessageReceiver<Intent, Effect>,
) {
    mpsc::unbounded_channel()
}

pub fn send_message<Intent: fmt::Debug, Effect: fmt::Debug>(
    message_tx: &MessageSender<Intent, Effect>,
    message: impl Into<Message<Intent, Effect>>,
) {
    let message = message.into();
    tracing::debug!("Sending message: {:?}", message);
    if let Err(message) = message_tx.send(message) {
        // Channel is closed, i.e. receiver has been dropped
        tracing::debug!("Failed to send message: {:?}", message.0);
    }
}

pub type RenderStateFn<State, Intent> = dyn FnMut(&State) -> Option<Intent> + Send;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message<Intent, Effect> {
    Intent(Intent),
    Effect(Effect),
}

impl<Intent, Effect> Message<Intent, Effect> {
    pub fn intent(intent: impl Into<Intent>) -> Self {
        Self::Intent(intent.into())
    }

    pub fn effect(effect: impl Into<Effect>) -> Self {
        Self::Effect(effect.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action<Effect, Task> {
    DispatchTask(Task),
    ApplyEffect(Effect),
}

impl<Effect, Task> Action<Effect, Task> {
    pub fn apply_effect(effect: impl Into<Effect>) -> Self {
        Self::ApplyEffect(effect.into())
    }

    pub fn dispatch_task(task: impl Into<Task>) -> Self {
        Self::DispatchTask(task.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageHandled {
    Progressing,
    NoProgress,
}

#[derive(Debug)]
pub struct PendingTasksCounter {
    number_of_pending_tasks: AtomicUsize,
}

impl PendingTasksCounter {
    pub const fn new() -> Self {
        Self {
            number_of_pending_tasks: AtomicUsize::new(0),
        }
    }
}

impl PendingTasksCounter {
    pub fn start_pending_task(&self) {
        self.number_of_pending_tasks
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        debug_assert!(!self.all_pending_tasks_finished());
    }

    pub fn finish_pending_task(&self) {
        debug_assert!(!self.all_pending_tasks_finished());
        self.number_of_pending_tasks
            .fetch_sub(1, std::sync::atomic::Ordering::Release);
    }

    pub fn all_pending_tasks_finished(&self) -> bool {
        self.number_of_pending_tasks
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }
}
