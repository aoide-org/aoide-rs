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

use super::{Action, ModelUpdate, State, Task};

use aoide_core::{collection::Collection, entity::EntityUid};

#[derive(Debug)]
pub enum Intent {
    CreateNewCollection(Collection),
    FetchAvailableCollections,
    ActivateCollection(Option<EntityUid>),
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> ModelUpdate {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::CreateNewCollection(new_collection) => ModelUpdate::unchanged(
                Action::dispatch_task(Task::CreateNewCollection(new_collection)),
            ),
            Self::FetchAvailableCollections => {
                state.remote.available_collections.set_pending_now();
                ModelUpdate::maybe_changed(Action::dispatch_task(Task::FetchAvailableCollections))
            }
            Self::ActivateCollection(new_active_collection_uid) => {
                state.set_active_collection_uid(new_active_collection_uid);
                ModelUpdate::maybe_changed(None)
            }
        }
    }
}
