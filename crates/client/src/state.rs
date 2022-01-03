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

use std::ops::{Add, AddAssign};

use crate::{action::Action, message::Message};

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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
