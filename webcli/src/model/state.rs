// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Effect, Intent, Message, StateUpdated, Task};

use aoide_client::{
    models::{collection, media_source, media_tracker},
    state::State as ClientState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ControlState {
    Running,
    Terminating,
}

impl ControlState {
    pub(super) const fn default() -> Self {
        Self::Running
    }
}

impl Default for ControlState {
    fn default() -> Self {
        Self::default()
    }
}

#[derive(Debug, Default)]
pub struct State {
    pub(super) last_errors: Vec<anyhow::Error>,
    pub(super) control_state: ControlState,
    pub active_collection: collection::State,
    pub media_sources: media_source::State,
    pub media_tracker: media_tracker::State,
}

impl State {
    pub fn last_errors(&self) -> &[anyhow::Error] {
        &self.last_errors
    }

    pub fn is_pending(&self) -> bool {
        self.active_collection.remote_view().is_pending()
            || self.media_sources.remote_view().is_pending()
            || self.media_tracker.remote_view().is_pending()
    }

    pub fn is_terminating(&self) -> bool {
        self.control_state == ControlState::Terminating
    }
}

impl ClientState for State {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn update(&mut self, message: Message) -> StateUpdated {
        log::debug!("Updating state {self:?} with message {message:?}");
        match message {
            Message::Intent(intent) => intent.apply_on(self),
            Message::Effect(effect) => effect.apply_on(self),
        }
    }
}
