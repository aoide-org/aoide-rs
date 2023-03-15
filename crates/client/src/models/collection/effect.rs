// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Entity, EntityUid};

use crate::util::roundtrip::PendingToken;

use super::{Action, EffectApplied, Model, PendingTask, Task};

#[derive(Debug)]
pub enum Effect {
    ActiveEntityUidUpdated {
        entity_uid: Option<EntityUid>,
    },
    PendingTaskAccepted {
        task: PendingTask,
    },
    FetchAllKindsFinished {
        token: PendingToken,
        result: anyhow::Result<Vec<String>>,
    },
    FetchFilteredEntitiesFinished {
        token: PendingToken,
        filtered_by_kind: Option<String>,
        result: anyhow::Result<Vec<Entity>>,
    },
    CreateEntityFinished(anyhow::Result<Entity>),
    UpdateEntityFinished(anyhow::Result<Entity>),
    PurgeEntityFinished(anyhow::Result<EntityUid>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, model: &mut Model) -> EffectApplied {
        log::trace!("Applying effect {self:?} on {model:?}");
        match self {
            Self::ActiveEntityUidUpdated { entity_uid } => {
                debug_assert!(!model.remote_view().is_pending());
                if model.active_entity_uid() == entity_uid.as_ref() {
                    // Nothing to do
                    return EffectApplied::unchanged_done();
                }
                model.set_active_entity_uid(entity_uid);
                EffectApplied::maybe_changed_done()
            }
            Self::PendingTaskAccepted { task } => {
                debug_assert!(!model.remote_view().is_pending());
                let token = model.remote_view.all_kinds.start_pending_now();
                let task = Task::Pending { token, task };
                let next_action = Action::dispatch_task(task);
                EffectApplied::maybe_changed(next_action)
            }
            Self::FetchAllKindsFinished { token, result } => match result {
                Ok(all_kinds) => {
                    if model.finish_pending_all_kinds(token, Some(all_kinds)) {
                        EffectApplied::maybe_changed_done()
                    } else {
                        EffectApplied::unchanged_done()
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    if model.finish_pending_all_kinds(token, None) {
                        EffectApplied::maybe_changed(next_action)
                    } else {
                        EffectApplied::unchanged(next_action)
                    }
                }
            },
            Self::FetchFilteredEntitiesFinished {
                token,
                filtered_by_kind,
                result,
            } => match result {
                Ok(filtered_entities) => {
                    if model.finish_pending_filtered_entities(
                        token,
                        filtered_by_kind,
                        Some(filtered_entities),
                    ) {
                        EffectApplied::maybe_changed_done()
                    } else {
                        EffectApplied::unchanged_done()
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    if model.finish_pending_filtered_entities(token, filtered_by_kind, None) {
                        EffectApplied::maybe_changed(next_action)
                    } else {
                        EffectApplied::unchanged(next_action)
                    }
                }
            },
            Self::CreateEntityFinished(res) | Self::UpdateEntityFinished(res) => match res {
                Ok(entity) => {
                    let next_action = model.after_entity_created_or_updated(entity);
                    if next_action.is_some() {
                        EffectApplied::maybe_changed(next_action)
                    } else {
                        EffectApplied::unchanged(next_action)
                    }
                }
                Err(err) => {
                    EffectApplied::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
                }
            },
            Self::PurgeEntityFinished(res) => match res {
                Ok(entity_uid) => {
                    let next_action = model.after_entity_purged(&entity_uid);
                    if next_action.is_some() {
                        EffectApplied::maybe_changed(next_action)
                    } else {
                        EffectApplied::unchanged(next_action)
                    }
                }
                Err(err) => {
                    EffectApplied::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
                }
            },
            Self::ErrorOccurred(error) => {
                EffectApplied::unchanged(Action::apply_effect(Self::ErrorOccurred(error)))
            }
        }
    }
}
