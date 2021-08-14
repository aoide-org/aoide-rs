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

use crate::prelude::remote::RemoteData;

use super::{Action, ControlState, State, StateUpdated, Task};

use aoide_core::usecases::media::tracker::{
    import::Outcome as ImportOutcome, scan::Outcome as ScanOutcome,
    untrack::Outcome as UntrackOutcome, Progress, Status,
};

#[derive(Debug)]
pub enum Effect {
    ProgressFetched(anyhow::Result<Progress>),
    Aborted(anyhow::Result<()>),
    StatusFetched(anyhow::Result<Status>),
    ScanFinished(anyhow::Result<ScanOutcome>),
    ImportFinished(anyhow::Result<ImportOutcome>),
    Untracked(anyhow::Result<UntrackOutcome>),
    PurgedUntracked(anyhow::Result<usize>),
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
            Self::ScanFinished(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state.remote_view.last_scan_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_scan_outcome = RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_scan_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::ImportFinished(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state.remote_view.last_import_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_import_outcome = RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_import_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::Untracked(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state.remote_view.last_untrack_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_untrack_outcome = RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_untrack_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::PurgedUntracked(res) => {
                debug_assert_eq!(state.control_state, ControlState::Busy);
                state.control_state = ControlState::Idle;
                state.remote_view.progress.reset();
                state.remote_view.status.reset();
                debug_assert!(state.remote_view.last_purge_untracked_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_purge_untracked_outcome =
                            RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote_view.last_purge_untracked_outcome.reset();
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
