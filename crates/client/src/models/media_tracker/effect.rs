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

use aoide_core_api::media::tracker::{
    find_untracked_files::Outcome as FindUntrackedFilesOutcome,
    import_files::Outcome as ImportFilesOutcome,
    scan_directories::Outcome as ScanDirectoriesOutcome,
    untrack_directories::Outcome as UntrackDirectoriesOutcome, Progress, Status,
};

use crate::prelude::remote::RemoteData;

use super::{Action, ControlState, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Effect {
    ProgressFetched(anyhow::Result<Progress>),
    Aborted(anyhow::Result<()>),
    StatusFetched(anyhow::Result<Status>),
    ScanDirectoriesFinished(anyhow::Result<ScanDirectoriesOutcome>),
    ImportFilesFinished(anyhow::Result<ImportFilesOutcome>),
    FindUntrackedFilesFinished(anyhow::Result<FindUntrackedFilesOutcome>),
    UntrackedDirectories(anyhow::Result<UntrackDirectoriesOutcome>),
    PurgedOrphanedAndUntracked(anyhow::Result<()>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ProgressFetched(res) => match res {
                Ok(new_progress) => {
                    let new_progress = RemoteData::ready_now(new_progress);
                    if state.remote_view.progress != new_progress {
                        state.remote_view.progress = new_progress;
                        StateUpdated::maybe_changed(None)
                    } else {
                        StateUpdated::unchanged(None)
                    }
                }
                Err(err) => StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::Aborted(res) => {
                let next_action = match res {
                    Ok(()) => Action::dispatch_task(Task::FetchProgress),
                    Err(err) => Action::apply_effect(Self::ErrorOccurred(err)),
                };
                StateUpdated::unchanged(next_action)
            }
            Self::StatusFetched(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                match res {
                    Ok(new_status) => {
                        let new_status = RemoteData::ready_now(new_status);
                        if state.remote_view.status != new_status {
                            StateUpdated::maybe_changed(None)
                        } else {
                            StateUpdated::unchanged(None)
                        }
                    }
                    Err(err) => {
                        StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
                    }
                }
            }
            Self::ScanDirectoriesFinished(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state.remote_view.last_scan_directories_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_scan_directories_outcome =
                            RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_scan_directories_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::ImportFilesFinished(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state.remote_view.last_import_files_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_import_files_outcome =
                            RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_import_files_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::FindUntrackedFilesFinished(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state
                    .remote_view
                    .last_find_untracked_files_outcome
                    .is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_find_untracked_files_outcome =
                            RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_find_untracked_files_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::UntrackedDirectories(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state
                    .remote_view
                    .last_untrack_directories_outcome
                    .is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_untrack_directories_outcome =
                            RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_untrack_directories_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::PurgedOrphanedAndUntracked(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state
                    .remote_view
                    .last_purge_orphaned_and_untracked_outcome
                    .is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_purge_orphaned_and_untracked_outcome =
                            RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state
                            .remote_view
                            .last_purge_orphaned_and_untracked_outcome
                            .reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::ErrorOccurred(err) => {
                StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
            }
        }
    }
}
