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

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    PurgeOrphaned {
        collection_uid: EntityUid,
        params: aoide_core_api::media::source::purge_orphaned::Params,
    },
    PurgeUntracked {
        collection_uid: EntityUid,
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
                if !state
                    .remote_view
                    .last_purge_orphaned_outcome
                    .try_set_pending_now()
                {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::PurgeOrphaned {
                            collection_uid,
                            params,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::maybe_changed(Action::dispatch_task(Task::PurgeOrphaned {
                    collection_uid,
                    params,
                }))
            }
            Self::PurgeUntracked {
                collection_uid,
                params,
            } => {
                if !state
                    .remote_view
                    .last_purge_untracked_outcome
                    .try_set_pending_now()
                {
                    log::warn!(
                        "Discarding intent while pending: {:?}",
                        Self::PurgeUntracked {
                            collection_uid,
                            params,
                        }
                    );
                    return StateUpdated::unchanged(None);
                }
                StateUpdated::maybe_changed(Action::dispatch_task(Task::PurgeUntracked {
                    collection_uid,
                    params,
                }))
            }
        }
    }
}
