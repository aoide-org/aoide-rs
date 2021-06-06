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

use std::{
    fmt,
    future::Future,
    ops::{Add, AddAssign},
    sync::{atomic::AtomicUsize, Arc},
    time::Instant,
};

use async_trait::async_trait;
use reqwest::{Client, Url};
use tokio::{signal, sync::mpsc};

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
pub struct MutableModelUpdated<E, T> {
    pub state_mutation: ModelMutation,
    pub next_action: Option<Action<E, T>>,
}

impl<E, T> MutableModelUpdated<E, T> {
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

pub fn mutable_model_updated<E1, E2, T1, T2>(
    from: MutableModelUpdated<E1, T1>,
) -> MutableModelUpdated<E2, T2>
where
    E1: Into<E2>,
    T1: Into<T2>,
{
    let MutableModelUpdated {
        state_mutation,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    MutableModelUpdated {
        state_mutation,
        next_action,
    }
}

pub trait MutableModel {
    type Intent;
    type Effect;
    type Task;

    fn update(
        &mut self,
        message: Message<Self::Intent, Self::Effect>,
    ) -> MutableModelUpdated<Self::Effect, Self::Task>;
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct DataSnapshot<T> {
    pub since: Instant,
    pub value: T,
}

impl<T> DataSnapshot<T> {
    pub fn new(since: impl Into<Instant>, value: impl Into<T>) -> Self {
        Self {
            since: since.into(),
            value: value.into(),
        }
    }

    pub fn now(value: impl Into<T>) -> Self {
        Self {
            since: Instant::now(),
            value: value.into(),
        }
    }

    pub fn as_ref(&self) -> DataSnapshot<&T> {
        let Self { since, value } = self;
        DataSnapshot {
            since: *since,
            value: &value,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RemoteData<T> {
    Unknown,
    Pending {
        stale_snapshot: DataSnapshot<Option<DataSnapshot<T>>>,
    },
    Ready {
        snapshot: Option<DataSnapshot<T>>,
    },
}

impl<T> Default for RemoteData<T> {
    fn default() -> Self {
        Self::Unknown
    }
}

impl<T> RemoteData<T> {
    pub fn ready_since(since: impl Into<Instant>, value: impl Into<T>) -> Self {
        Self::Ready {
            snapshot: Some(DataSnapshot::new(since, value)),
        }
    }

    pub fn ready_now(value: impl Into<T>) -> Self {
        Self::Ready {
            snapshot: Some(DataSnapshot::now(value)),
        }
    }

    pub fn get(&self) -> Option<&DataSnapshot<T>> {
        match self {
            Self::Unknown => None,
            Self::Pending { stale_snapshot } => stale_snapshot.value.as_ref(),
            Self::Ready { snapshot } => snapshot.as_ref(),
        }
    }

    pub fn get_ready(&self) -> Option<&DataSnapshot<T>> {
        match self {
            Self::Unknown | Self::Pending { .. } => None,
            Self::Ready { snapshot } => snapshot.as_ref(),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut DataSnapshot<T>> {
        match self {
            Self::Unknown | Self::Pending { .. } => None,
            Self::Ready { snapshot } => snapshot.as_mut(),
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Unknown;
    }

    pub fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }

    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending { .. })
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn take_ready(&mut self) -> Option<DataSnapshot<T>> {
        if let Self::Ready { snapshot } = self {
            let snapshot = snapshot.take();
            debug_assert!(snapshot.is_some());
            *self = Self::Unknown;
            snapshot
        } else {
            None
        }
    }

    pub fn set_pending_since(&mut self, since: impl Into<Instant>) {
        let stale_snapshot = DataSnapshot {
            since: since.into(),
            value: self.take_ready(),
        };
        debug_assert!(
            stale_snapshot.since
                >= stale_snapshot
                    .value
                    .as_ref()
                    .map(|x| x.since)
                    .unwrap_or(stale_snapshot.since)
        );
        *self = Self::Pending { stale_snapshot };
    }

    pub fn set_pending_now(&mut self) {
        self.set_pending_since(Instant::now());
    }
}

#[derive(Debug, Clone)]
pub enum Message<I, E> {
    Intent(I),
    Effect(E),

    /// Special intent to initiate a graceful shutdown
    IntentTerminate,

    /// Special effect to signal that the message loop can now
    /// be exited safely
    EffectTerminated,
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
pub enum MessageLoopControl {
    Continue,
    Terminate,
}

pub type RenderMutableModelFn<M, I> = dyn FnMut(&M) -> Option<I> + Send;

pub fn handle_next_message<M>(
    shared_env: &Arc<Environment>,
    message_tx: Option<&MessageSender<M::Intent, M::Effect>>,
    model: &mut M,
    render_fn: &mut RenderMutableModelFn<M, M::Intent>,
    mut next_message: Message<M::Intent, M::Effect>,
) -> MessageLoopControl
where
    M: MutableModel + fmt::Debug,
    M::Intent: fmt::Debug + Send + 'static,
    M::Effect: fmt::Debug + Send + 'static,
    M::Task: AsyncTask<M::Effect> + fmt::Debug + 'static,
{
    let mut state_mutation = ModelMutation::Unchanged;
    let mut number_of_next_actions = 0;
    let mut number_of_messages_sent = 0;
    let mut number_of_tasks_dispatched = 0;
    'process_next_message: loop {
        let MutableModelUpdated {
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
                Action::DispatchTask(task) => {
                    if let Some(message_tx) = message_tx {
                        log::debug!("Dispatching task asynchronously: {:?}", task);
                        Environment::dispatch_task(
                            shared_env.clone(),
                            message_tx.clone(),
                            task.execute(shared_env.clone()),
                        );
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
        if state_mutation == ModelMutation::MaybeChanged || number_of_next_actions > 0 {
            log::debug!("Rendering current state: {:?}", model);
            if let Some(rendering_intent) = render_fn(&model) {
                if let Some(message_tx) = message_tx {
                    log::debug!(
                        "Received intent after rendering state: {:?}",
                        rendering_intent
                    );
                    send_message(&message_tx, Message::Intent(rendering_intent));
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

pub async fn message_loop<M>(
    shared_env: Arc<Environment>,
    initial_model: M,
    first_message: impl Into<Message<M::Intent, M::Effect>>,
    mut render_fn: Box<RenderMutableModelFn<M, M::Intent>>,
) -> M
where
    M: MutableModel + fmt::Debug,
    M::Intent: fmt::Debug + Send + 'static,
    M::Effect: fmt::Debug + Send + 'static,
    M::Task: AsyncTask<M::Effect> + fmt::Debug + 'static,
{
    let mut model = initial_model;
    // If needed the message channel could be allocated in the outer context
    // to allow sending of messages from external sources. But then implicit
    // termination after all message senders have been dropped depends on
    // those outstanding, external references!
    let (message_tx, mut message_rx) = message_channel();
    // Kick off the loop by sending a first message
    send_message(&message_tx, first_message);
    let mut message_tx = Some(message_tx);
    let mut terminating = false;
    loop {
        tokio::select! {
            Some(next_message) = message_rx.recv() => {
                if terminating && matches!(next_message, Message::EffectTerminated) {
                    log::debug!("Exiting message loop after terminated received");
                    break;
                }
                match handle_next_message(&shared_env, message_tx.as_ref(), &mut model, &mut *render_fn, next_message) {
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
                log::info!("Terminating after receiving SIGINT...");
                terminating = true;
                debug_assert!(message_tx.is_some());
                send_message(message_tx.as_ref().unwrap(), Message::IntentTerminate);
            }
            else => {
                // Exit the message loop if message_rx.recv() returned None
                debug_assert!(message_rx.recv().await.is_none());
                log::debug!("Exiting message loop after all message senders have been dropped");
                break;
            }
        }
    }
    debug_assert!(terminating);
    debug_assert!(message_tx.is_none());
    model
}
