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

use crate::prelude::*;

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    FetchProgress,
    FetchStatus {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::query_status::Params,
    },
    StartScanDirectories {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::scan_directories::Params,
    },
    StartImportFiles {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::import_files::Params,
    },
    StartFindUntrackedFiles {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::find_untracked_files::Params,
    },
    UntrackDirectories {
        collection_uid: CollectionUid,
        params: aoide_core_api::media::tracker::untrack_directories::Params,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {self:?} on {state:?}");
        match self {
            Self::FetchProgress => {
                let token = state.remote_view.progress.start_pending_now();
                let task = Task::FetchProgress { token };
                log::debug!("Dispatching task {task:?}");
                StateUpdated::maybe_changed(Action::dispatch_task(task))
            }
            Self::FetchStatus {
                collection_uid,
                params,
            } => {
                let token = state.remote_view.progress.start_pending_now();
                let task = Task::FetchStatus {
                    token,
                    collection_uid,
                    params,
                };
                log::debug!("Dispatching task {task:?}");
                StateUpdated::maybe_changed(Action::dispatch_task(task))
            }
            Self::StartScanDirectories {
                collection_uid,
                params,
            } => {
                if let Some(token) = state
                    .remote_view
                    .last_scan_directories_outcome
                    .try_start_pending_now()
                {
                    let task = Task::StartScanDirectories {
                        token,
                        collection_uid,
                        params,
                    };
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::StartScanDirectories {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    StateUpdated::unchanged(None)
                }
            }
            Self::StartImportFiles {
                collection_uid,
                params,
            } => {
                if let Some(token) = state
                    .remote_view
                    .last_import_files_outcome
                    .try_start_pending_now()
                {
                    let task = Task::StartImportFiles {
                        token,
                        collection_uid,
                        params,
                    };
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::StartImportFiles {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    StateUpdated::unchanged(None)
                }
            }
            Self::StartFindUntrackedFiles {
                collection_uid,
                params,
            } => {
                if let Some(token) = state
                    .remote_view
                    .last_find_untracked_files_outcome
                    .try_start_pending_now()
                {
                    let task = Task::StartFindUntrackedFiles {
                        token,
                        collection_uid,
                        params,
                    };
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::StartFindUntrackedFiles {
                        collection_uid,
                        params,
                    };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    StateUpdated::unchanged(None)
                }
            }
            Self::UntrackDirectories {
                collection_uid,
                params,
            } => {
                if let Some(token) = state
                    .remote_view
                    .last_untrack_directories_outcome
                    .try_start_pending_now()
                {
                    let task = Task::UntrackDirectories {
                        token,
                        collection_uid,
                        params,
                    };
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::UntrackDirectories {
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
