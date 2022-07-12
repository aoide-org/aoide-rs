// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Entity, EntityUid};

use crate::util::roundtrip::PendingToken;

use super::{Action, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
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
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {self:?} on {state:?}");
        match self {
            Self::FetchAllKindsFinished { token, result } => match result {
                Ok(all_kinds) => {
                    let next_action = None;
                    if state.finish_pending_all_kinds(token, Some(all_kinds)) {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    if state.finish_pending_all_kinds(token, None) {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::FetchFilteredEntitiesFinished {
                token,
                filtered_by_kind,
                result,
            } => match result {
                Ok(filtered_entities) => {
                    let next_action = None;
                    if state.finish_pending_filtered_entities(
                        token,
                        filtered_by_kind,
                        Some(filtered_entities),
                    ) {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    if state.finish_pending_filtered_entities(token, filtered_by_kind, None) {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::CreateEntityFinished(res) => match res {
                Ok(entity) => {
                    let next_action = state.after_entity_created_or_updated(entity);
                    if next_action.is_some() {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
                Err(err) => StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::UpdateEntityFinished(res) => match res {
                Ok(entity) => {
                    let next_action = state.after_entity_created_or_updated(entity);
                    if next_action.is_some() {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
                Err(err) => StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::PurgeEntityFinished(res) => match res {
                Ok(entity_uid) => {
                    let next_action = state.after_entity_purged(&entity_uid);
                    if next_action.is_some() {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
                Err(err) => StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::ErrorOccurred(error) => {
                StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(error)))
            }
        }
    }
}
