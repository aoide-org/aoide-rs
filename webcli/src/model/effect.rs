// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::num::NonZeroUsize;

use aoide_client::models::{collection, media_source, media_tracker};
use aoide_core_api::track::find_unsynchronized::UnsynchronizedTrackEntity;
use infect::ModelChanged;

use super::{EffectApplied, Model};
use crate::model::{State, Task};

#[derive(Debug)]
pub enum Effect {
    ErrorOccurred(anyhow::Error),
    FirstErrorsDiscarded(NonZeroUsize),
    AbortFinished(anyhow::Result<()>),
    ActiveCollection(collection::Effect),
    MediaSources(media_source::Effect),
    MediaTracker(media_tracker::Effect),
    FindUnsynchronizedTracksFinished(anyhow::Result<Vec<UnsynchronizedTrackEntity>>),
    ExportTracksFinished(anyhow::Result<()>),
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
                EffectApplied::maybe_changed()
            }
            Self::FirstErrorsDiscarded(num_errors) => {
                debug_assert!(num_errors.get() <= model.last_errors.len());
                model.last_errors = model.last_errors.drain(num_errors.get()..).collect();
                EffectApplied::maybe_changed()
            }
            Self::AbortFinished(res) => {
                match res {
                    Ok(()) => {
                        if model.state == State::Terminating && model.is_pending() {
                            // Abort next pending request until idle
                            EffectApplied::unchanged_task(Task::AbortPendingRequest)
                        } else {
                            EffectApplied::unchanged()
                        }
                    }
                    Err(err) => {
                        model.last_errors.push(err);
                        EffectApplied::maybe_changed()
                    }
                }
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
                        EffectApplied::unchanged()
                    }
                    Err(err) => {
                        model.last_errors.push(err);
                        EffectApplied::maybe_changed()
                    }
                }
            }
            Self::ExportTracksFinished(res) => {
                if let Err(err) = res {
                    model.last_errors.push(err);
                    EffectApplied::maybe_changed()
                } else {
                    EffectApplied::unchanged()
                }
            }
            Self::AbortPendingRequest(state) => {
                let mut effect_applied = if model.abort_pending_request_effect().is_some() {
                    EffectApplied::unchanged_task(Task::AbortPendingRequest)
                } else {
                    EffectApplied::unchanged()
                };
                let Some(state) = state else {
                    return effect_applied;
                };
                if model.state == state {
                    return effect_applied;
                }
                model.state = state;
                effect_applied.render_hint = ModelChanged::MaybeChanged;
                effect_applied
            }
        }
    }
}
