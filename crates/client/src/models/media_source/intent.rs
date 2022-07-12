// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

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
        log::trace!("Applying intent {self:?} on {state:?}");
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
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::PurgeOrphaned {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
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
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::PurgeUntracked {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    StateUpdated::unchanged(None)
                }
            }
        }
    }
}
