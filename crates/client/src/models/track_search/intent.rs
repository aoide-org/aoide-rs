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

use aoide_core::entity::EntityUid;

use aoide_core_api::track::search::Params;

use super::{Action, FetchResultPageRequest, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    Reset(Option<Params>),
    FetchResultPage {
        collection_uid: EntityUid,
        request: FetchResultPageRequest,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        tracing::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::Reset(search_params) => {
                if !state.can_reset() {
                    tracing::warn!("Cannot fetch results: {:?}", search_params);
                    return StateUpdated::unchanged(None);
                }
                state.reset(search_params);
                StateUpdated::maybe_changed(None)
            }
            Self::FetchResultPage {
                collection_uid,
                request,
            } => {
                let task = Task::FetchResultPage {
                    collection_uid,
                    request,
                };
                if !state.can_fetch_results() {
                    tracing::warn!("Cannot fetch results: {:?}", task);
                    return StateUpdated::unchanged(None);
                }
                state.set_fetching_results();
                StateUpdated::maybe_changed(Action::dispatch_task(task))
            }
        }
    }
}
