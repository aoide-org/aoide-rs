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
    ops::{Add, AddAssign},
    sync::atomic::AtomicUsize,
    time::Instant,
};

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

    pub fn task_pending(&self) {
        self.pending_tasks_count
            .fetch_add(1, std::sync::atomic::Ordering::Acquire);
    }

    pub fn task_finished(&self) {
        self.pending_tasks_count
            .fetch_sub(1, std::sync::atomic::Ordering::Release);
    }

    pub fn all_tasks_finished(&self) -> bool {
        self.pending_tasks_count
            .load(std::sync::atomic::Ordering::Acquire)
            == 0
    }
}

pub type MessageSender<T> = mpsc::UnboundedSender<T>;
pub type MessageReceiver<T> = mpsc::UnboundedReceiver<T>;

pub fn message_channel<T>() -> (MessageSender<T>, MessageReceiver<T>) {
    mpsc::unbounded_channel()
}

pub fn send_message<T: fmt::Debug>(message_tx: &MessageSender<T>, message: impl Into<T>) {
    let message = message.into();
    log::debug!("Emitting message: {:?}", message);
    if let Err(message) = message_tx.send(message) {
        // Channel is closed, i.e. receiver has been dropped
        log::debug!("Failed to emit message: {:?}", message.0);
    }
}

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

pub fn message_applied<A, B>(
    (state_mutation, next_action): (StateMutation, Option<A>),
) -> (StateMutation, Option<B>)
where
    A: Into<B>,
{
    (state_mutation, next_action.map(Into::into))
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

#[derive(Debug)]
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
