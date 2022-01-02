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

use super::{Action, State, StateUpdated, Task};

use aoide_core::{collection::Collection, entity::EntityUid};

#[derive(Debug)]
pub enum Intent {
    CreateCollection { new_collection: Collection },
    FetchAvailableCollections,
    ActivateCollection { collection_uid: Option<EntityUid> },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::CreateCollection { new_collection } => {
                let task = Task::CreateCollection { new_collection };
                log::debug!("Dispatching task {:?}", task);
                StateUpdated::unchanged(Action::dispatch_task(task))
            }
            Self::FetchAvailableCollections => {
                if let Some(pending_counter) = state
                    .remote_view
                    .available_collections
                    .try_set_pending_now()
                {
                    let task = Task::FetchAvailableCollections { pending_counter };
                    log::debug!("Dispatching task {:?}", task);
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::FetchAvailableCollections;
                    log::warn!(
                        "Discarding intent while already pending: {:?}",
                        self_reconstructed
                    );
                    StateUpdated::unchanged(None)
                }
            }
            Self::ActivateCollection { collection_uid } => {
                state.set_active_collection_uid(collection_uid);
                StateUpdated::maybe_changed(None)
            }
        }
    }
}
