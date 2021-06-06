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

use crate::prelude::{RemoteData, StateMutation};

use super::{Action, ControlState, State, Task};

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
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ProgressFetched(res) => match res {
                Ok(new_progress) => {
                    let new_progress = RemoteData::ready_now(new_progress);
                    if state.remote.progress != new_progress {
                        state.remote.progress = new_progress;
                        (StateMutation::MaybeChanged, None)
                    } else {
                        (StateMutation::Unchanged, None)
                    }
                }
                Err(err) => (
                    StateMutation::Unchanged,
                    Some(Action::apply_effect(Self::ErrorOccurred(err))),
                ),
            },
            Self::Aborted(res) => {
                let next_action = match res {
                    Ok(()) => Action::dispatch_task(Task::FetchProgress),
                    Err(err) => Action::apply_effect(Self::ErrorOccurred(err)),
                };
                (StateMutation::Unchanged, Some(next_action))
            }
            Self::StatusFetched(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                match res {
                    Ok(new_status) => {
                        let new_status = RemoteData::ready_now(new_status);
                        if state.remote.status != new_status {
                            (StateMutation::MaybeChanged, None)
                        } else {
                            (StateMutation::Unchanged, None)
                        }
                    }
                    Err(err) => (
                        StateMutation::Unchanged,
                        Some(Action::apply_effect(Self::ErrorOccurred(err))),
                    ),
                }
            }
            Self::ScanFinished(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_scan_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_scan_outcome = RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote.last_scan_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                (StateMutation::MaybeChanged, Some(next_action))
            }
            Self::ImportFinished(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                // Invalidate both progress and status to enforce refetching
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_import_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_import_outcome = RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote.last_import_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                (StateMutation::MaybeChanged, Some(next_action))
            }
            Self::Untracked(res) => {
                debug_assert_eq!(state.control, ControlState::Busy);
                state.control = ControlState::Idle;
                state.remote.progress.reset();
                state.remote.status.reset();
                debug_assert!(state.remote.last_untrack_outcome.is_pending());
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote.last_untrack_outcome = RemoteData::ready_now(outcome);
                        Action::dispatch_task(Task::FetchProgress)
                    }
                    Err(err) => {
                        state.remote.last_untrack_outcome.reset();
                        Action::apply_effect(Self::ErrorOccurred(err))
                    }
                };
                (StateMutation::MaybeChanged, Some(next_action))
            }
            Self::ErrorOccurred(err) => (
                StateMutation::Unchanged,
                Some(Action::apply_effect(Self::ErrorOccurred(err))),
            ),
        }
    }
}
