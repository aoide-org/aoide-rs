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

use crate::prelude::round_counter::RoundCounter;

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Effect {
    FetchProgressFinished {
        pending_counter: RoundCounter,
        result: anyhow::Result<aoide_core_api::media::tracker::Progress>,
    },
    FetchStatusFinished {
        pending_counter: RoundCounter,
        result: anyhow::Result<aoide_core_api::media::tracker::Status>,
    },
    ScanDirectoriesFinished(
        anyhow::Result<aoide_core_api::media::tracker::scan_directories::Outcome>,
    ),
    UntrackDirectoriesFinished(
        anyhow::Result<aoide_core_api::media::tracker::untrack_directories::Outcome>,
    ),
    ImportFilesFinished(anyhow::Result<aoide_core_api::media::tracker::import_files::Outcome>),
    FindUntrackedFilesFinished(
        anyhow::Result<aoide_core_api::media::tracker::find_untracked_files::Outcome>,
    ),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::FetchProgressFinished {
                pending_counter,
                result,
            } => match result {
                Ok(progress) => {
                    let (finished, _) = state
                        .remote_view
                        .progress
                        .finish_pending_round_with_value_now(pending_counter, progress);
                    if finished {
                        StateUpdated::maybe_changed(None)
                    } else {
                        StateUpdated::unchanged(None)
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    let finished = state
                        .remote_view
                        .progress
                        .finish_pending_round(pending_counter);
                    if finished {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::FetchStatusFinished {
                pending_counter,
                result,
            } => match result {
                Ok(status) => {
                    let (finished, _) = state
                        .remote_view
                        .status
                        .finish_pending_round_with_value_now(pending_counter, status);
                    if finished {
                        StateUpdated::maybe_changed(None)
                    } else {
                        StateUpdated::unchanged(None)
                    }
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    let finished = state
                        .remote_view
                        .status
                        .finish_pending_round(pending_counter);
                    if finished {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::ScanDirectoriesFinished(res) => {
                if !state.remote_view.last_scan_directories_outcome.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::ScanDirectoriesFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                let next_action = match res {
                    Ok(outcome) => {
                        state
                            .remote_view
                            .last_scan_directories_outcome
                            .finish_pending_round_with_value_now(
                                state
                                    .remote_view
                                    .last_scan_directories_outcome
                                    .round_counter(),
                                outcome,
                            );
                        let pending_counter = state.remote_view.progress.set_pending_now();
                        Action::dispatch_task(Task::FetchProgress { pending_counter })
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
                let next_action = match res {
                    Ok(outcome) => {
                        state
                            .remote_view
                            .last_untrack_directories_outcome
                            .finish_pending_round_with_value_now(
                                state
                                    .remote_view
                                    .last_untrack_directories_outcome
                                    .round_counter(),
                                outcome,
                            );
                        let pending_counter = state.remote_view.progress.set_pending_now();
                        Action::dispatch_task(Task::FetchProgress { pending_counter })
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
                let next_action = match res {
                    Ok(outcome) => {
                        state
                            .remote_view
                            .last_import_files_outcome
                            .finish_pending_round_with_value_now(
                                state.remote_view.last_import_files_outcome.round_counter(),
                                outcome,
                            );
                        let pending_counter = state.remote_view.progress.set_pending_now();
                        Action::dispatch_task(Task::FetchProgress { pending_counter })
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
                        state
                            .remote_view
                            .last_find_untracked_files_outcome
                            .finish_pending_round_with_value_now(
                                state
                                    .remote_view
                                    .last_find_untracked_files_outcome
                                    .round_counter(),
                                outcome,
                            );
                        let pending_counter = state.remote_view.progress.set_pending_now();
                        Action::dispatch_task(Task::FetchProgress { pending_counter })
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
