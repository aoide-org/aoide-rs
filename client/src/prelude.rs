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

pub type EventSender<T> = mpsc::UnboundedSender<T>;
pub type EventReceiver<T> = mpsc::UnboundedReceiver<T>;

pub fn event_channel<T>() -> (EventSender<T>, EventReceiver<T>) {
    mpsc::unbounded_channel()
}

pub fn emit_event<T: fmt::Debug>(event_tx: &EventSender<T>, event: impl Into<T>) {
    let event = event.into();
    log::debug!("Emitting event: {:?}", event);
    if let Err(event) = event_tx.send(event) {
        // Channel is closed, i.e. receiver has been dropped
        log::debug!("Failed to emit event: {:?}", event.0);
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

pub fn event_applied<A, B>(
    (state_mutation, next_action): (StateMutation, Option<A>),
) -> (StateMutation, Option<B>)
where
    A: Into<B>,
{
    (state_mutation, next_action.map(Into::into))
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RemoteData<T> {
    Unknown,
    Pending { stale_data: Option<T> },
    Ready { data: Option<T> },
}

impl<T> Default for RemoteData<T> {
    fn default() -> Self {
        Self::Unknown
    }
}

impl<T> RemoteData<T> {
    pub fn ready(data: T) -> Self {
        Self::Ready { data: Some(data) }
    }

    pub fn get(&self) -> Option<&T> {
        match self {
            Self::Unknown => None,
            Self::Pending { stale_data } => stale_data.as_ref(),
            Self::Ready { data } => data.as_ref(),
        }
    }

    pub fn get_ready(&self) -> Option<&T> {
        match self {
            Self::Unknown | Self::Pending { .. } => None,
            Self::Ready { data } => data.as_ref(),
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Unknown | Self::Pending { .. } => None,
            Self::Ready { data } => data.as_mut(),
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

    pub fn take_ready(&mut self) -> Option<T> {
        if let Self::Ready { data } = self {
            let data = data.take();
            debug_assert!(data.is_some());
            *self = Self::Unknown;
            data
        } else {
            None
        }
    }

    pub fn set_pending(&mut self) {
        let stale_data = self.take_ready();
        *self = Self::Pending { stale_data }
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
