// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, time::Instant};

use aoide_client::{
    models::{collection, media_source, media_tracker},
    state::state_updated,
};

use crate::model::state::ControlState;

use super::{Action, CollectionUid, Effect, ExportTracksParams, State, StateUpdated, Task};

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
    FindUnsynchronizedTracks {
        collection_uid: CollectionUid,
        params: aoide_core_api::track::find_unsynchronized::Params,
    },
    ExportTracks {
        collection_uid: CollectionUid,
        params: ExportTracksParams,
    },
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
        log::debug!("Applying intent {self:?} on {state:?}");
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
                    log::debug!("Discarding intent while not running: {self_reconstructed:?}");
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
                            "Discarding only {num_errors} instead of {num_errors_requested} errors"
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
                    log::debug!("Discarding intent while not running: {intent:?}");
                    return StateUpdated::unchanged(None);
                }
                state_updated(intent.apply_on(&mut state.active_collection))
            }
            Self::MediaSources(intent) => {
                if state.control_state != ControlState::Running {
                    log::debug!("Discarding intent while not running: {intent:?}");
                    return StateUpdated::unchanged(None);
                }
                state_updated(intent.apply_on(&mut state.media_sources))
            }
            Self::MediaTracker(intent) => {
                if state.control_state != ControlState::Running {
                    log::debug!("Discarding intent while not running: {intent:?}");
                    return StateUpdated::unchanged(None);
                }
                state_updated(intent.apply_on(&mut state.media_tracker))
            }
            Self::FindUnsynchronizedTracks {
                collection_uid,
                params,
            } => {
                let next_action = Action::dispatch_task(Task::FindUnsynchronizedTracks {
                    collection_uid,
                    params,
                });
                StateUpdated::unchanged(next_action)
            }
            Self::ExportTracks {
                collection_uid,
                params,
            } => {
                let next_action = Action::dispatch_task(Task::ExportTracks {
                    collection_uid,
                    params,
                });
                StateUpdated::unchanged(next_action)
            }
        }
    }
}

fn abort_pending_request_action(state: &State) -> Option<Action> {
    state
        .is_pending()
        .then(|| Action::dispatch_task(Task::AbortPendingRequest))
}
