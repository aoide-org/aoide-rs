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

use super::{Action, State, StateUpdate};

use aoide_core::collection::Entity as CollectionEntity;

#[derive(Debug)]
pub enum Effect {
    NewCollectionCreated(anyhow::Result<CollectionEntity>),
    AvailableCollectionsFetched(anyhow::Result<Vec<CollectionEntity>>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdate {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::NewCollectionCreated(res) => match res {
                Ok(_) => StateUpdate::unchanged(None),
                Err(err) => StateUpdate::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::AvailableCollectionsFetched(res) => match res {
                Ok(new_available_collections) => {
                    state.set_available_collections(new_available_collections);
                    StateUpdate::maybe_changed(None)
                }
                Err(err) => StateUpdate::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::ErrorOccurred(error) => {
                StateUpdate::unchanged(Action::apply_effect(Self::ErrorOccurred(error)))
            }
        }
    }
}
