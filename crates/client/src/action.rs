// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
