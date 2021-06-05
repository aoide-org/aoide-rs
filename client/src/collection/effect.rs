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

use crate::prelude::StateMutation;

use super::{Action, State};

use aoide_core::collection::Entity as CollectionEntity;

#[derive(Debug)]
pub enum Effect {
    NewCollectionCreated(anyhow::Result<CollectionEntity>),
    AvailableCollectionsFetched(anyhow::Result<Vec<CollectionEntity>>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying event {:?} on {:?}", self, state);
        match self {
            Self::NewCollectionCreated(res) => match res {
                Ok(_) => (StateMutation::Unchanged, None),
                Err(err) => (
                    StateMutation::Unchanged,
                    Some(Action::apply_effect(Self::ErrorOccurred(err))),
                ),
            },
            Self::AvailableCollectionsFetched(res) => match res {
                Ok(new_available_collections) => {
                    state.set_available_collections(new_available_collections);
                    (StateMutation::MaybeChanged, None)
                }
                Err(err) => (
                    StateMutation::Unchanged,
                    Some(Action::apply_effect(Self::ErrorOccurred(err))),
                ),
            },
            Self::ErrorOccurred(error) => (
                StateMutation::Unchanged,
                Some(Action::apply_effect(Self::ErrorOccurred(error))),
            ),
        }
    }
}
