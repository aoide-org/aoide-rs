// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Entity, EntityUid};

use super::{EffectApplied, Model};
use crate::util::roundtrip::PendingToken;

#[derive(Debug)]
pub enum Effect {
    ActiveEntityUidUpdated {
        entity_uid: Option<EntityUid>,
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
            Self::FetchAllKindsFinished { token, result } => match result {
                Ok(all_kinds) => {
                    if model.finish_pending_all_kinds(token, Some(all_kinds)) {
                        EffectApplied::maybe_changed_done()
                    } else {
                        EffectApplied::unchanged_done()
                    }
                }
                Err(err) => {
                    model.last_error = Some(err);
                    model.finish_pending_all_kinds(token, None);
                    EffectApplied::maybe_changed_done()
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
                    model.last_error = Some(err);
                    model.finish_pending_filtered_entities(token, filtered_by_kind, None);
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::CreateEntityFinished(res) | Self::UpdateEntityFinished(res) => match res {
                Ok(entity) => model.after_entity_created_or_updated(entity),
                Err(err) => {
                    model.last_error = Some(err);
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::PurgeEntityFinished(res) => match res {
                Ok(entity_uid) => model.after_entity_purged(&entity_uid),
                Err(err) => {
                    model.last_error = Some(err);
                    EffectApplied::maybe_changed_done()
                }
            },
            Self::ErrorOccurred(err) => {
                model.last_error = Some(err);
                EffectApplied::maybe_changed_done()
            }
        }
    }
}
