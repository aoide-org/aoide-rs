// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::ops::{Add, AddAssign};

use crate::{action::Action, message::Message};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StateUpdated<Effect, Task> {
    pub state_mutation: StateMutation,
    pub next_action: Option<Action<Effect, Task>>,
}

impl<Effect, Task> StateUpdated<Effect, Task> {
    pub fn unchanged(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            state_mutation: StateMutation::Unchanged,
            next_action: next_action.into(),
        }
    }

    pub fn maybe_changed(next_action: impl Into<Option<Action<Effect, Task>>>) -> Self {
        Self {
            state_mutation: StateMutation::MaybeChanged,
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
        state_mutation,
        next_action,
    } = from;
    let next_action = next_action.map(|action| match action {
        Action::ApplyEffect(effect) => Action::apply_effect(effect),
        Action::DispatchTask(task) => Action::dispatch_task(task),
    });
    StateUpdated {
        state_mutation,
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
