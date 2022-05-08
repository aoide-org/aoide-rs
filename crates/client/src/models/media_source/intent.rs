// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

use super::{Action, CollectionUid, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    PurgeOrphaned {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::source::purge_orphaned::Params,
    },
    PurgeUntracked {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::source::purge_untracked::Params,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::PurgeOrphaned {
                collection_uid,
                params,
            } => {
                if let Some(token) = state
                    .remote_view
                    .last_purge_orphaned_outcome
                    .try_start_pending_now()
                {
                    let task = Task::PurgeOrphaned {
                        token,
                        collection_uid,
                        params,
                    };
                    log::debug!("Dispatching task {:?}", task);
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::PurgeOrphaned {
                        collection_uid,
                        params,
                    };
                    log::warn!(
                        "Discarding intent while already pending: {:?}",
                        self_reconstructed
                    );
                    StateUpdated::unchanged(None)
                }
            }
            Self::PurgeUntracked {
                collection_uid,
                params,
            } => {
                if let Some(token) = state
                    .remote_view
                    .last_purge_untracked_outcome
                    .try_start_pending_now()
                {
                    let task = Task::PurgeUntracked {
                        token,
                        collection_uid,
                        params,
                    };
                    log::debug!("Dispatching task {:?}", task);
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::PurgeUntracked {
                        collection_uid,
                        params,
                    };
                    log::warn!(
                        "Discarding intent while already pending: {:?}",
                        self_reconstructed
                    );
                    StateUpdated::unchanged(None)
                }
            }
        }
    }
}
