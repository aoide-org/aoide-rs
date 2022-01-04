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

use aoide_core::collection::Entity as CollectionEntity;

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
        result: anyhow::Result<Vec<CollectionEntity>>,
    },
    CreateEntityFinished(anyhow::Result<CollectionEntity>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
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
                    let next_action = state.after_entity_created(entity);
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
