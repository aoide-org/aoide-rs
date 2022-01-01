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

use super::{Effect, Intent, Message, StateUpdated, Task};

use crate::{
    models::{active_collection, media_sources, media_tracker},
    prelude::mutable::State as MutableState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    Running,
    Terminating,
}

impl ControlState {
    pub const fn default() -> Self {
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
    pub active_collection: active_collection::State,
    pub media_sources: media_sources::State,
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

impl MutableState for State {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn update(&mut self, message: Message) -> StateUpdated {
        log::debug!("Updating state {:?} with message {:?}", self, message);
        match message {
            Message::Intent(intent) => intent.apply_on(self),
            Message::Effect(effect) => effect.apply_on(self),
        }
    }
}