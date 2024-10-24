// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::{Add, AddAssign};

use discro::Publisher;
use tokio::task::JoinHandle;

pub use aoide_backend_embedded::Environment;

pub mod fs;

/// Collection management
pub mod collection;

/// Settings management
pub mod settings;

/// Track management
pub mod track;

#[derive(Debug)]
pub enum JoinedTask<T> {
    Completed(T),
    Cancelled,
    Panicked(anyhow::Error),
}

impl<T> JoinedTask<T> {
    pub async fn join(handle: JoinHandle<T>) -> Self {
        match handle.await {
            Ok(output) => Self::Completed(output),
            Err(err) => {
                if err.is_cancelled() {
                    Self::Cancelled
                } else {
                    debug_assert!(err.is_panic());
                    Self::Panicked(err.into())
                }
            }
        }
    }
}

impl<T> From<T> for JoinedTask<T> {
    fn from(completed: T) -> Self {
        Self::Completed(completed)
    }
}

#[derive(Debug)]
pub enum Reaction {
    Rejected,
    Accepted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateEffect {
    /// The state has not been modified.
    Unchanged,
    /// The state might have been modified.
    MaybeChanged,
    /// The state has been modified.
    Changed,
}

impl Add for StateEffect {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Unchanged, Self::Unchanged) => Self::Unchanged,
            (Self::MaybeChanged, Self::Unchanged | Self::MaybeChanged)
            | (Self::Unchanged, Self::MaybeChanged) => Self::MaybeChanged,
            (Self::Changed, _) | (_, Self::Changed) => Self::Changed,
        }
    }
}

impl AddAssign for StateEffect {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

#[derive(Debug)]
pub struct StateUnchanged;

impl From<StateUnchanged> for StateEffect {
    fn from(_: StateUnchanged) -> Self {
        StateEffect::Unchanged
    }
}

pub(crate) fn modify_shared_state<S>(
    shared_state: &Publisher<S>,
    modify: impl FnOnce(&mut S) -> (anyhow::Result<Reaction>, StateEffect),
) -> (anyhow::Result<Reaction>, StateEffect) {
    let mut reaction_result = Ok(Reaction::Rejected);
    let mut state_effect = StateEffect::Unchanged;
    shared_state.modify(|state| {
        let (modify_reaction_result, modify_state_effect) = modify(state);
        reaction_result = modify_reaction_result;
        state_effect = modify_state_effect;
        match state_effect {
            StateEffect::Unchanged => false,
            StateEffect::MaybeChanged | StateEffect::Changed => true,
        }
    });
    (reaction_result, state_effect)
}

pub(crate) fn modify_shared_state_infallible<S>(
    shared_state: &Publisher<S>,
    modify: impl FnOnce(&mut S) -> (Reaction, StateEffect),
) -> (Reaction, StateEffect) {
    let mut reaction = Reaction::Rejected;
    let mut state_effect = StateEffect::Unchanged;
    shared_state.modify(|state| {
        let (modify_reaction, modify_state_effect) = modify(state);
        reaction = modify_reaction;
        state_effect = modify_state_effect;
        match state_effect {
            StateEffect::Unchanged => false,
            StateEffect::MaybeChanged | StateEffect::Changed => true,
        }
    });
    (reaction, state_effect)
}
