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

use crate::{
    models::{active_collection, media_sources, media_tracker, webcli::state::ControlState},
    prelude::mutable::state_updated,
};

use super::{Action, Effect, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    RenderState,
    TimedIntent {
        not_before: Instant,
        intent: Box<Intent>,
    },
    InjectEffect(Box<Effect>),
    DiscardFirstErrors(NonZeroUsize),
    ActiveCollection(active_collection::Intent),
    MediaSources(media_sources::Intent),
    MediaTracker(media_tracker::Intent),
    AbortPendingRequest,
    Terminate,
}

impl From<active_collection::Intent> for Intent {
    fn from(intent: active_collection::Intent) -> Self {
        Self::ActiveCollection(intent)
    }
}

impl From<media_sources::Intent> for Intent {
    fn from(intent: media_sources::Intent) -> Self {
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
            Self::RenderState => StateUpdated::maybe_changed(None),
            Self::TimedIntent { not_before, intent } => {
                if state.control_state != ControlState::Running {
                    log::debug!("Discarding timed intent while not running: {:?}", intent);
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::unchanged(Action::dispatch_task(Task::TimedIntent {
                    not_before,
                    intent,
                }))
            }
            Self::InjectEffect(effect) => StateUpdated::unchanged(Action::apply_effect(*effect)),
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
            Self::AbortPendingRequest => {
                StateUpdated::unchanged(abort_pending_request_action(state))
            }
            Self::Terminate => {
                if state.control_state == ControlState::Terminating {
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Terminating;
                StateUpdated::maybe_changed(abort_pending_request_action(state))
            }
        }
    }
}

fn abort_pending_request_action(state: &State) -> Option<Action> {
    state
        .is_pending()
        .then(|| Action::dispatch_task(Task::AbortPendingRequest))
}
