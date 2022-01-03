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

use crate::util::roundtrip::PendingWatermark;

use super::{Action, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    CreateCollectionFinished(anyhow::Result<CollectionEntity>),
    FetchAvailableCollectionsFinished {
        token: PendingWatermark,
        result: anyhow::Result<Vec<CollectionEntity>>,
    },
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::CreateCollectionFinished(res) => match res {
                Ok(_) => StateUpdated::unchanged(None),
                Err(err) => StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::FetchAvailableCollectionsFinished { token, result } => match result {
                Ok(available_collections) => {
                    let next_action = None;
                    if state
                        .finish_pending_available_collections(token, Some(available_collections))
                    {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    if state.finish_pending_available_collections(token, None) {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::ErrorOccurred(error) => {
                StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(error)))
            }
        }
    }
}
