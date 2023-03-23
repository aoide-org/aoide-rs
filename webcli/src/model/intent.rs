// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{num::NonZeroUsize, time::Instant};

use aoide_client::models::{collection, media_source, media_tracker};

use super::{CollectionUid, Effect, EffectApplied, ExportTracksParams, IntentHandled, Model, Task};
use crate::model::State;

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
    pub fn handle_on(self, model: &mut Model) -> IntentHandled {
        log::debug!("Applying {self:?} on {model:?}");
        match self {
            Self::RenderModel => {
                // Enforce re-rendering by considering the unchanged model as (maybe) changed
                IntentHandled::Accepted(EffectApplied::maybe_changed_done())
            }
            Self::Schedule { not_before, intent } => {
                if model.state != State::Running {
                    let self_reconstructed = Self::Schedule { not_before, intent };
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let task = Task::ScheduleIntent { not_before, intent };
                IntentHandled::Accepted(EffectApplied::unchanged(task))
            }
            Self::DiscardFirstErrors(num_errors_requested) => {
                let Some(num_errors) =
                    NonZeroUsize::new(num_errors_requested.get().min(model.last_errors.len())) else {
                        return IntentHandled::Rejected(Self::DiscardFirstErrors(num_errors_requested));
                    };
                if num_errors < num_errors_requested {
                    debug_assert!(num_errors_requested.get() > 1);
                    log::debug!(
                        "Discarding only {num_errors} instead of {num_errors_requested} errors"
                    );
                }
                let effect = Effect::FirstErrorsDiscarded(num_errors);
                IntentHandled::Accepted(effect.apply_on(model))
            }
            Self::AbortPendingRequest => {
                let Some(effect) = model.abort_pending_request_effect() else {
                    return IntentHandled::Rejected(Self::AbortPendingRequest);
                };
                IntentHandled::Accepted(effect.apply_on(model))
            }
            Self::Terminate => {
                if model.state != State::Running {
                    return IntentHandled::Rejected(Self::Terminate);
                }
                let effect = Effect::AbortPendingRequest(Some(State::Terminating));
                IntentHandled::Accepted(effect.apply_on(model))
            }
            Self::ActiveCollection(intent) => {
                if model.state != State::Running {
                    return IntentHandled::Rejected(Self::ActiveCollection(intent));
                }
                intent.handle_on(&mut model.active_collection).map_into()
            }
            Self::MediaSources(intent) => {
                if model.state != State::Running {
                    return IntentHandled::Rejected(Self::MediaSources(intent));
                }
                intent.handle_on(&mut model.media_sources).map_into()
            }
            Self::MediaTracker(intent) => {
                if model.state != State::Running {
                    return IntentHandled::Rejected(Self::MediaTracker(intent));
                }
                intent.handle_on(&mut model.media_tracker).map_into()
            }
            Self::FindUnsynchronizedTracks {
                collection_uid,
                params,
            } => {
                let task = Task::FindUnsynchronizedTracks {
                    collection_uid,
                    params,
                };
                IntentHandled::Accepted(EffectApplied::unchanged(task))
            }
            Self::ExportTracks {
                collection_uid,
                params,
            } => {
                let task = Task::ExportTracks {
                    collection_uid,
                    params,
                };
                IntentHandled::Accepted(EffectApplied::unchanged(task))
            }
        }
    }
}
