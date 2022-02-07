// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use aoide_client::{
    models::{collection, media_source, media_tracker},
    state::state_updated,
};
use aoide_core_api::track::find_unsynchronized::UnsynchronizedTrackEntity;

use crate::model::{state::ControlState, Action, Task};

use super::{Intent, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
    FirstErrorsDiscarded(NonZeroUsize),
    ApplyIntent(Intent),
    AbortFinished(anyhow::Result<()>),
    ActiveCollection(collection::Effect),
    MediaSources(media_source::Effect),
    MediaTracker(media_tracker::Effect),
    FindUnsynchronizedTracksFinished(anyhow::Result<Vec<UnsynchronizedTrackEntity>>),
    ExportTracksFinished(anyhow::Result<()>),
}

impl From<collection::Effect> for Effect {
    fn from(effect: collection::Effect) -> Self {
        Self::ActiveCollection(effect)
    }
}

impl From<media_source::Effect> for Effect {
    fn from(effect: media_source::Effect) -> Self {
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
            | Self::ActiveCollection(collection::Effect::ErrorOccurred(error))
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
            Self::ActiveCollection(effect) => {
                state_updated(effect.apply_on(&mut state.active_collection))
            }
            Self::MediaSources(effect) => state_updated(effect.apply_on(&mut state.media_sources)),
            Self::MediaTracker(effect) => state_updated(effect.apply_on(&mut state.media_tracker)),
            Self::FindUnsynchronizedTracksFinished(res) => {
                match res {
                    Ok(entities) => {
                        // TODO: Store received entities in state
                        for entity in entities {
                            log::info!("{:?}", entity);
                        }
                        StateUpdated::unchanged(None)
                    }
                    Err(err) => {
                        state.last_errors.push(err);
                        StateUpdated::maybe_changed(None)
                    }
                }
            }
            Self::ExportTracksFinished(res) => {
                if let Err(err) = res {
                    state.last_errors.push(err);
                    StateUpdated::maybe_changed(None)
                } else {
                    StateUpdated::unchanged(None)
                }
            }
        }
    }
}
