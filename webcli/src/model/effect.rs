// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroUsize;

use infect::IntentHandled;

use aoide_client::models::{collection, media_source, media_tracker};
use aoide_core_api::track::find_unsynchronized::UnsynchronizedTrackEntity;

use crate::model::{Action, State, Task};

use super::{EffectApplied, Intent, Model};

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
    FirstErrorsDiscarded(NonZeroUsize),
    HandleIntent(Intent),
    AbortFinished(anyhow::Result<()>),
    ActiveCollection(collection::Effect),
    MediaSources(media_source::Effect),
    MediaTracker(media_tracker::Effect),
    FindUnsynchronizedTracksFinished(anyhow::Result<Vec<UnsynchronizedTrackEntity>>),
    ExportTracksFinished(anyhow::Result<()>),
    RenderModel,
    AbortPendingRequest(Option<State>),
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
    pub fn apply_on(self, model: &mut Model) -> EffectApplied {
        log::debug!("Applying {self:?} on {model:?}");
        match self {
            Self::ErrorOccurred(error)
            | Self::ActiveCollection(collection::Effect::ErrorOccurred(error))
            | Self::MediaTracker(media_tracker::Effect::ErrorOccurred(error)) => {
                model.last_errors.push(error);
                EffectApplied::maybe_changed_done()
            }
            Self::FirstErrorsDiscarded(num_errors) => {
                debug_assert!(num_errors.get() <= model.last_errors.len());
                model.last_errors = model.last_errors.drain(num_errors.get()..).collect();
                EffectApplied::maybe_changed_done()
            }
            Self::HandleIntent(intent) => {
                let next_action = match intent.apply_on(model) {
                    IntentHandled::Accepted(next_action) => next_action,
                    IntentHandled::Rejected(_) => None,
                };
                EffectApplied::unchanged(next_action)
            }
            Self::AbortFinished(res) => {
                let next_action = match res {
                    Ok(()) => {
                        if model.state == State::Terminating && model.is_pending() {
                            // Abort next pending request until idle
                            Some(Action::SpawnTask(Task::AbortPendingRequest))
                        } else {
                            None
                        }
                    }
                    Err(err) => Some(Action::apply_effect(Self::ErrorOccurred(err))),
                };
                EffectApplied::unchanged(next_action)
            }
            Self::ActiveCollection(effect) => {
                effect.apply_on(&mut model.active_collection).map_into()
            }
            Self::MediaSources(effect) => effect.apply_on(&mut model.media_sources).map_into(),
            Self::MediaTracker(effect) => effect.apply_on(&mut model.media_tracker).map_into(),
            Self::FindUnsynchronizedTracksFinished(res) => {
                match res {
                    Ok(entities) => {
                        // TODO: Store received entities in model
                        for entity in entities {
                            log::info!("{entity:?}");
                        }
                        EffectApplied::unchanged_done()
                    }
                    Err(err) => {
                        model.last_errors.push(err);
                        EffectApplied::maybe_changed_done()
                    }
                }
            }
            Self::ExportTracksFinished(res) => {
                if let Err(err) = res {
                    model.last_errors.push(err);
                    EffectApplied::maybe_changed_done()
                } else {
                    EffectApplied::unchanged_done()
                }
            }
            Self::RenderModel => EffectApplied::maybe_changed_done(), // enforce re-rendering
            Self::AbortPendingRequest(state) => {
                let next_action = model.abort_pending_request_action();
                let Some(state) = state else {
                    return EffectApplied::unchanged(next_action);
                };
                if model.state == state {
                    return EffectApplied::unchanged(next_action);
                }
                model.state = state;
                EffectApplied::maybe_changed(next_action)
            }
        }
    }
}
