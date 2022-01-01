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

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Effect {
    FetchProgressFinished(anyhow::Result<Progress>),
    FetchStatusFinished(anyhow::Result<Status>),
    ScanDirectoriesFinished(anyhow::Result<ScanDirectoriesOutcome>),
    UntrackDirectoriesFinished(anyhow::Result<UntrackDirectoriesOutcome>),
    ImportFilesFinished(anyhow::Result<ImportFilesOutcome>),
    FindUntrackedFilesFinished(anyhow::Result<FindUntrackedFilesOutcome>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::FetchProgressFinished(res) => {
                if !state.remote_view.progress.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::FetchProgressFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                match res {
                    Ok(new_progress) => {
                        let new_progress = RemoteData::ready_now(new_progress);
                        if state.remote_view.progress != new_progress {
                            state.remote_view.progress = new_progress;
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
            Self::FetchStatusFinished(res) => {
                if !state.remote_view.status.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::FetchStatusFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
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
                if !state.remote_view.last_scan_directories_outcome.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::ScanDirectoriesFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
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
            Self::UntrackDirectoriesFinished(res) => {
                if !state
                    .remote_view
                    .last_untrack_directories_outcome
                    .is_pending()
                {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::UntrackDirectoriesFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
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
            Self::ImportFilesFinished(res) => {
                if !state.remote_view.last_import_files_outcome.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::ImportFilesFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
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
                if !state
                    .remote_view
                    .last_find_untracked_files_outcome
                    .is_pending()
                {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::FindUntrackedFilesFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
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
            Self::ErrorOccurred(err) => {
                StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
            }
        }
    }
}
