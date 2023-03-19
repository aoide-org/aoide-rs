// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, time::Instant};

use aoide_client::models::{collection, media_source, media_tracker};

use crate::model::State;

use super::{Action, CollectionUid, Effect, ExportTracksParams, IntentHandled, Model, Task};

#[derive(Debug)]
pub enum Intent {
    RenderModel,
    Schedule {
        not_before: Instant,
        intent: Box<Intent>,
    },
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
    #[must_use]
    pub fn apply_on(self, model: &Model) -> IntentHandled {
        log::debug!("Applying {self:?} on {model:?}");
        match self {
            Self::RenderModel => {
                IntentHandled::Accepted(Action::apply_effect(Effect::RenderModel).into())
            }
            Self::Schedule { not_before, intent } => {
                if model.state == State::Running {
                    let next_action =
                        Action::spawn_task(Task::ScheduleIntent { not_before, intent });
                    IntentHandled::accepted(next_action)
                } else {
                    let self_reconstructed = Self::Schedule { not_before, intent };
                    log::debug!("Discarding intent while not running: {self_reconstructed:?}");
                    IntentHandled::Rejected(self_reconstructed)
                }
            }
            Self::DiscardFirstErrors(num_errors_requested) => {
                let num_errors =
                    NonZeroUsize::new(num_errors_requested.get().min(model.last_errors.len()));
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
                IntentHandled::Accepted(next_action)
            }
            Self::AbortPendingRequest => {
                let next_action = model.abort_pending_request_action();
                IntentHandled::Accepted(next_action)
            }
            Self::Terminate => {
                if model.state == State::Terminating {
                    // Already terminating, nothing to do
                    return IntentHandled::Accepted(None);
                }
                let next_action =
                    Action::apply_effect(Effect::AbortPendingRequest(Some(State::Terminating)));
                IntentHandled::accepted(next_action)
            }
            Self::ActiveCollection(intent) => {
                if model.state != State::Running {
                    let self_reconstructed = Self::ActiveCollection(intent);
                    log::debug!("Discarding intent while not running: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                intent.apply_on(&model.active_collection).map_into()
            }
            Self::MediaSources(intent) => {
                if model.state != State::Running {
                    let self_reconstructed = Self::MediaSources(intent);
                    log::debug!("Discarding intent while not running: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                intent.apply_on(&model.media_sources).map_into()
            }
            Self::MediaTracker(intent) => {
                if model.state != State::Running {
                    let self_reconstructed = Self::MediaTracker(intent);
                    log::debug!("Discarding intent while not running: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                intent.apply_on(&model.media_tracker).map_into()
            }
            Self::FindUnsynchronizedTracks {
                collection_uid,
                params,
            } => {
                let next_action = Action::spawn_task(Task::FindUnsynchronizedTracks {
                    collection_uid,
                    params,
                });
                IntentHandled::accepted(next_action)
            }
            Self::ExportTracks {
                collection_uid,
                params,
            } => {
                let next_action = Action::spawn_task(Task::ExportTracks {
                    collection_uid,
                    params,
                });
                IntentHandled::accepted(next_action)
            }
        }
    }
}
