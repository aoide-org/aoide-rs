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

use std::num::NonZeroUsize;

use crate::{
    models::{
        active_collection, media_sources, media_tracker,
        webcli::{state::ControlState, Action, Task},
    },
    prelude::mutable::state_updated,
};

use super::{Intent, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
    FirstErrorsDiscarded(NonZeroUsize),
    ApplyIntent(Intent),
    ActiveCollection(active_collection::Effect),
    MediaSources(media_sources::Effect),
    MediaTracker(media_tracker::Effect),
    AbortFinished(anyhow::Result<()>),
}

impl From<active_collection::Effect> for Effect {
    fn from(effect: active_collection::Effect) -> Self {
        Self::ActiveCollection(effect)
    }
}

impl From<media_sources::Effect> for Effect {
    fn from(effect: media_sources::Effect) -> Self {
        Self::MediaSources(effect)
    }
}

impl From<media_tracker::Effect> for Effect {
    fn from(effect: media_tracker::Effect) -> Self {
        Self::MediaTracker(effect)
    }
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::debug!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ErrorOccurred(error)
            | Self::ActiveCollection(active_collection::Effect::ErrorOccurred(error))
            | Self::MediaTracker(media_tracker::Effect::ErrorOccurred(error)) => {
                state.last_errors.push(error);
                StateUpdated::maybe_changed(None)
            }
            Self::FirstErrorsDiscarded(num_errors) => {
                debug_assert!(num_errors.get() <= state.last_errors.len());
                state.last_errors = state.last_errors.drain(num_errors.get()..).collect();
                StateUpdated::maybe_changed(None)
            }
            Self::ApplyIntent(intent) => intent.apply_on(state),
            Self::ActiveCollection(effect) => {
                state_updated(effect.apply_on(&mut state.active_collection))
            }
            Self::MediaSources(effect) => state_updated(effect.apply_on(&mut state.media_sources)),
            Self::MediaTracker(effect) => state_updated(effect.apply_on(&mut state.media_tracker)),
            Self::AbortFinished(res) => {
                let next_action = match res {
                    Ok(()) => {
                        if state.control_state == ControlState::Terminating && state.is_pending() {
                            // Abort next pending request until idle
                            Some(Action::DispatchTask(Task::AbortPendingRequest))
                        } else {
                            None
                        }
                    }
                    Err(err) => Some(Action::apply_effect(Self::ErrorOccurred(err))),
                };
                StateUpdated::unchanged(next_action)
            }
        }
    }
}