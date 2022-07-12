// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::util::roundtrip::PendingToken;

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Effect {
    FetchProgressFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::Progress>,
    },
    FetchStatusFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::Status>,
    },
    ScanDirectoriesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::scan_directories::Outcome>,
    },
    UntrackDirectoriesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::untrack_directories::Outcome>,
    },
    ImportFilesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::import_files::Outcome>,
    },
    FindUntrackedFilesFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::tracker::find_untracked_files::Outcome>,
    },
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {self:?} on {state:?}");
        match self {
            Self::FetchProgressFinished { token, result } => match result {
                Ok(progress) => {
                    if let Err(outcome) = state
                        .remote_view
                        .progress
                        .finish_pending_with_value_now(token, progress)
                    {
                        let effect_reconstructed = Self::FetchProgressFinished {
                            token,
                            result: Ok(outcome),
                        };
                        // Doesn't matter when fetching data
                        log::debug!("Discarding outdated effect: {effect_reconstructed:?}");
                        return StateUpdated::unchanged(None);
                    }
                    StateUpdated::maybe_changed(None)
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    let finished = state.remote_view.progress.finish_pending(token);
                    if finished {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::FetchStatusFinished { token, result } => match result {
                Ok(status) => {
                    if let Err(outcome) = state
                        .remote_view
                        .status
                        .finish_pending_with_value_now(token, status)
                    {
                        let effect_reconstructed = Self::FetchStatusFinished {
                            token,
                            result: Ok(outcome),
                        };
                        // Doesn't matter when fetching data
                        log::debug!("Discarding outdated effect: {effect_reconstructed:?}");
                        return StateUpdated::unchanged(None);
                    }
                    StateUpdated::maybe_changed(None)
                }
                Err(err) => {
                    let next_action = Action::apply_effect(Self::ErrorOccurred(err));
                    let finished = state.remote_view.status.finish_pending(token);
                    if finished {
                        StateUpdated::maybe_changed(next_action)
                    } else {
                        StateUpdated::unchanged(next_action)
                    }
                }
            },
            Self::ScanDirectoriesFinished { token, result } => {
                let next_action = match result {
                    Ok(outcome) => {
                        if let Err(outcome) = state
                            .remote_view
                            .last_scan_directories_outcome
                            .finish_pending_with_value_now(token, outcome)
                        {
                            let effect_reconstructed = Self::ScanDirectoriesFinished {
                                token,
                                result: Ok(outcome),
                            };
                            log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                            return StateUpdated::unchanged(None);
                        }
                        let token = state.remote_view.progress.start_pending_now();
                        Action::dispatch_task(Task::FetchProgress { token })
                    }
                    Err(err) => {
                        state.remote_view.last_scan_directories_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::UntrackDirectoriesFinished { token, result } => {
                let next_action = match result {
                    Ok(outcome) => {
                        if let Err(outcome) = state
                            .remote_view
                            .last_untrack_directories_outcome
                            .finish_pending_with_value_now(token, outcome)
                        {
                            let effect_reconstructed = Self::UntrackDirectoriesFinished {
                                token,
                                result: Ok(outcome),
                            };
                            log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                            return StateUpdated::unchanged(None);
                        }
                        let token = state.remote_view.progress.start_pending_now();
                        Action::dispatch_task(Task::FetchProgress { token })
                    }
                    Err(err) => {
                        state.remote_view.last_untrack_directories_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::ImportFilesFinished { token, result } => {
                let next_action = match result {
                    Ok(outcome) => {
                        if let Err(outcome) = state
                            .remote_view
                            .last_import_files_outcome
                            .finish_pending_with_value_now(token, outcome)
                        {
                            let effect_reconstructed = Self::ImportFilesFinished {
                                token,
                                result: Ok(outcome),
                            };
                            log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                            return StateUpdated::unchanged(None);
                        }
                        let token = state.remote_view.progress.start_pending_now();
                        Action::dispatch_task(Task::FetchProgress { token })
                    }
                    Err(err) => {
                        state.remote_view.last_import_files_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::FindUntrackedFilesFinished { token, result } => {
                let next_action = match result {
                    Ok(outcome) => {
                        if let Err(outcome) = state
                            .remote_view
                            .last_find_untracked_files_outcome
                            .finish_pending_with_value_now(token, outcome)
                        {
                            let effect_reconstructed = Self::FindUntrackedFilesFinished {
                                token,
                                result: Ok(outcome),
                            };
                            log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                            return StateUpdated::unchanged(None);
                        }
                        let token = state.remote_view.progress.start_pending_now();
                        Action::dispatch_task(Task::FetchProgress { token })
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
