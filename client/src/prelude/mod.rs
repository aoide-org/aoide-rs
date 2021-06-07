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
    future::Future,
    sync::{atomic::AtomicUsize, Arc},
};

use async_trait::async_trait;
use reqwest::{Client, Url};
use tokio::sync::mpsc;

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

    pub fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_count
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }

    pub fn dispatch_task<I, T>(
        shared_self: Arc<Self>,
        message_tx: MessageSender<I, T::Output>,
        task: T,
    ) where
        T: Future + Send + 'static,
        T::Output: fmt::Debug + Send + 'static,
        I: fmt::Debug + Send + 'static,
    {
        shared_self
            .pending_tasks_count
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
        tokio::spawn(async move {
            let effect = task.await;
            log::debug!("Received effect from task: {:?}", effect);
            send_message(&message_tx, Message::Effect(effect));
            shared_self
                .pending_tasks_count
                .fetch_sub(1, std::sync::atomic::Ordering::Release);
        });
    }
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

#[async_trait]
pub trait AsyncTask<E> {
    async fn execute(self, shared_env: Arc<Environment>) -> E;
}

pub type RenderModelFn<M, I> = dyn FnMut(&M) -> Option<I> + Send;

#[derive(Debug, Clone)]
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
pub(crate) enum MessageLoopControl {
    Continue,
    Terminate,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MessageLoopState {
    Running,
    Terminating,
}
