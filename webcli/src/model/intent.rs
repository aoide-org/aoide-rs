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

use std::{num::NonZeroUsize, time::Instant};

use aoide_client::{
    models::{collection, media_source, media_tracker},
    state::state_updated,
};

use crate::model::state::ControlState;

use super::{Action, Effect, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    RenderState,
    Deferred {
        not_before: Instant,
        intent: Box<Intent>,
    },
    InjectEffect(Box<Effect>),
    DiscardFirstErrors(NonZeroUsize),
    AbortPendingRequest,
    Terminate,
    ActiveCollection(collection::Intent),
    MediaSources(media_source::Intent),
    MediaTracker(media_tracker::Intent),
}

impl From<collection::Intent> for Intent {
    fn from(intent: collection::Intent) -> Self {
        Self::ActiveCollection(intent)
    }
}

impl From<media_source::Intent> for Intent {
    fn from(intent: media_source::Intent) -> Self {
        Self::MediaSources(intent)
    }
}

impl From<media_tracker::Intent> for Intent {
    fn from(intent: media_tracker::Intent) -> Self {
        Self::MediaTracker(intent)
    }
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::debug!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::RenderState => StateUpdated::maybe_changed(None), // enfore re-rendering
            Self::Deferred { not_before, intent } => {
                let next_action = if state.control_state == ControlState::Running {
                    Some(Action::dispatch_task(Task::DeferredIntent {
                        not_before,
                        intent,
                    }))
                } else {
                    let self_reconstructed = Self::Deferred { not_before, intent };
                    log::debug!(
                        "Discarding intent while not running: {:?}",
                        self_reconstructed
                    );
                    None
                };
                StateUpdated::unchanged(next_action)
            }
            Self::InjectEffect(effect) => {
                let next_action = Action::apply_effect(*effect);
                StateUpdated::unchanged(next_action)
            }
            Self::DiscardFirstErrors(num_errors_requested) => {
                let num_errors =
                    NonZeroUsize::new(num_errors_requested.get().min(state.last_errors.len()));
                let next_action = if let Some(num_errors) = num_errors {
                    if num_errors < num_errors_requested {
                        debug_assert!(num_errors_requested.get() > 1);
                        log::debug!(
                            "Discarding only {} instead of {} errors",
                            num_errors,
                            num_errors_requested
                        );
                    }
                    Some(Action::apply_effect(Effect::FirstErrorsDiscarded(
                        num_errors,
                    )))
                } else {
                    log::debug!("No errors to discard");
                    None
                };
                StateUpdated::unchanged(next_action)
            }
            Self::AbortPendingRequest => {
                let next_action = abort_pending_request_action(state);
                StateUpdated::unchanged(next_action)
            }
            Self::Terminate => {
                if state.control_state == ControlState::Terminating {
                    // Already terminating, nothing to do
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Terminating;
                let next_action = abort_pending_request_action(state);
                StateUpdated::maybe_changed(next_action)
            }
            Self::ActiveCollection(intent) => {
                if state.control_state != ControlState::Running {
                    log::debug!("Discarding intent while not running: {:?}", intent);
                    return StateUpdated::unchanged(None);
                }
                state_updated(intent.apply_on(&mut state.active_collection))
            }
            Self::MediaSources(intent) => {
                if state.control_state != ControlState::Running {
                    log::debug!("Discarding intent while not running: {:?}", intent);
                    return StateUpdated::unchanged(None);
                }
                state_updated(intent.apply_on(&mut state.media_sources))
            }
            Self::MediaTracker(intent) => {
                if state.control_state != ControlState::Running {
                    log::debug!("Discarding intent while not running: {:?}", intent);
                    return StateUpdated::unchanged(None);
                }
                state_updated(intent.apply_on(&mut state.media_tracker))
            }
        }
    }
}

fn abort_pending_request_action(state: &State) -> Option<Action> {
    state
        .is_pending()
        .then(|| Action::dispatch_task(Task::AbortPendingRequest))
}
