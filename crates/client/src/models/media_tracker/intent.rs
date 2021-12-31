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

use aoide_core::{entity::EntityUid, util::url::BaseUrl};

use super::{Action, ControlState, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    FetchProgress,
    Abort,
    AbortOnTermination,
    StartScanDirectories {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    StartImportFiles {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    StartFindUntrackedFiles {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
    UntrackDirectories {
        collection_uid: EntityUid,
        root_url: BaseUrl,
    },
    PurgeOrphanedAndUntracked {
        collection_uid: EntityUid,
        root_url: Option<BaseUrl>,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::FetchProgress => {
                state.remote_view.progress.set_pending_now();
                StateUpdated::unchanged(Action::dispatch_task(Task::FetchProgress))
            }
            Self::Abort => StateUpdated::unchanged(Action::dispatch_task(Task::Abort)),
            Self::AbortOnTermination => {
                if state.control_state != ControlState::Idle {
                    // Only dispatch an abort task if a local task is pending
                    StateUpdated::unchanged(Action::dispatch_task(Task::Abort))
                } else {
                    // Nothing to do
                    StateUpdated::unchanged(None)
                }
            }
            Self::FetchStatus {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot fetch status while not idle");
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Busy;
                state.remote_view.status.set_pending_now();
                StateUpdated::maybe_changed(Action::dispatch_task(Task::FetchStatus {
                    collection_uid,
                    root_url,
                }))
            }
            Self::StartScanDirectories {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start scan while not idle");
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Busy;
                state.remote_view.progress.reset();
                state.remote_view.status.set_pending_now();
                state
                    .remote_view
                    .last_scan_directories_outcome
                    .set_pending_now();
                StateUpdated::maybe_changed(Action::dispatch_task(Task::StartScanDirectories {
                    collection_uid,
                    root_url,
                }))
            }
            Self::StartImportFiles {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start import while not idle");
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Busy;
                state.remote_view.progress.reset();
                state.remote_view.status.set_pending_now();
                state
                    .remote_view
                    .last_import_files_outcome
                    .set_pending_now();
                StateUpdated::maybe_changed(Action::dispatch_task(Task::StartImportFiles {
                    collection_uid,
                    root_url,
                }))
            }
            Self::StartFindUntrackedFiles {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start finding untracked entries while not idle");
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Busy;
                state.remote_view.progress.reset();
                state.remote_view.status.set_pending_now();
                state
                    .remote_view
                    .last_find_untracked_files_outcome
                    .set_pending_now();
                StateUpdated::maybe_changed(Action::dispatch_task(Task::StartFindUntrackedFiles {
                    collection_uid,
                    root_url,
                }))
            }
            Self::UntrackDirectories {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot untrack while not idle");
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Busy;
                state.remote_view.progress.reset();
                state.remote_view.status.set_pending_now();
                state
                    .remote_view
                    .last_untrack_directories_outcome
                    .set_pending_now();
                StateUpdated::maybe_changed(Action::dispatch_task(Task::UntrackDirectories {
                    collection_uid,
                    root_url: Some(root_url),
                }))
            }
            Self::PurgeOrphanedAndUntracked {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot purge untracked while not idle");
                    return StateUpdated::unchanged(None);
                }
                state.control_state = ControlState::Busy;
                state.remote_view.progress.reset();
                state.remote_view.status.set_pending_now();
                state
                    .remote_view
                    .last_purge_orphaned_and_untracked_outcome
                    .set_pending_now();
                StateUpdated::maybe_changed(Action::dispatch_task(Task::Purge {
                    collection_uid,
                    root_url,
                }))
            }
        }
    }
}
