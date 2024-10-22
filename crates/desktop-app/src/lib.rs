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

#[must_use]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionEffect {
    /// The state of the action's target is unchanged.
    ///
    /// The action has been rejected or has no effect.
    ///
    /// Must not be returned if the state has changed.
    Unchanged,
    /// The state of the action's target might have changed.
    ///
    /// If unsure use this variant. In this case the caller must account for any effect,
    /// both unchanged and changed.
    MaybeChanged,
    /// The state of the action's target has changed.
    ///
    /// Must not be returned if the state is unchanged.
    Changed,
}

impl Add for ActionEffect {
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

impl AddAssign for ActionEffect {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

#[derive(Debug)]
pub struct ActionRejected;

#[derive(Debug)]
pub struct StateUnchanged;

impl From<StateUnchanged> for ActionEffect {
    fn from(_: StateUnchanged) -> Self {
        ActionEffect::Unchanged
    }
}

pub(crate) fn modify_shared_state_action_effect<State>(
    shared_state: &Publisher<State>,
    modify_state: impl FnOnce(&mut State) -> ActionEffect,
) -> ActionEffect {
    let mut effect = ActionEffect::MaybeChanged;
    shared_state.modify(|state| {
        effect = modify_state(state);
        match effect {
            ActionEffect::Unchanged => false,
            ActionEffect::MaybeChanged | ActionEffect::Changed => true,
        }
    });
    effect
}

pub(crate) fn modify_shared_state_action_effect_result<State, Result>(
    shared_state: &Publisher<State>,
    modify_state: impl FnOnce(&mut State) -> (ActionEffect, Result),
) -> (ActionEffect, Result) {
    let mut effect = ActionEffect::MaybeChanged;
    let mut result = None;
    shared_state.modify(|state| {
        let (modify_effect, modify_result) = modify_state(state);
        effect = modify_effect;
        result = Some(modify_result);
        match modify_effect {
            ActionEffect::Unchanged => false,
            ActionEffect::MaybeChanged | ActionEffect::Changed => true,
        }
    });
    let result = result.expect("has been set");
    (effect, result)
}
