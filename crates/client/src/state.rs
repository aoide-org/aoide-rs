// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::{Add, AddAssign};

use crate::{action::Action, message::Message};

/// Perceptible effect when updating the state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateChanged {
    /// The state has not changed
    Unchanged,

    /// The state might have changed
    ///
    /// False positives are allowed, i.e. when unsure or when determining
    /// if the state has actually changed is either costly or impossible
    /// then default to this variant.
    MaybeChanged,
}

impl Add<StateChanged> for StateChanged {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Unchanged, Self::Unchanged) => Self::Unchanged,
            (_, _) => Self::MaybeChanged,
        }
    }
}

impl AddAssign for StateChanged {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateUpdated<Effect, Task> {
    /// The outcome on the state itself
    ///
    /// The state might have been modified during by the update
    /// operation.
    pub changed: StateChanged,

    /// The next action
    ///
    /// Updating the state results in 0 or 1 next action(s).
    pub next_action: Option<Action<Effect, Task>>,
}

impl<Effect, Task> StateUpdated<Effect, Task> {
    pub fn unchanged(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            changed: StateChanged::Unchanged,
            next_action: next_action.into(),
        }
    }

    pub fn maybe_changed(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            changed: StateChanged::MaybeChanged,
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
        changed,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    StateUpdated {
        changed,
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

pub type RenderStateFn<State, Intent> = dyn FnMut(&State) -> Option<Intent> + Send;
