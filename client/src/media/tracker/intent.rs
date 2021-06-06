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

use crate::prelude::*;

use super::{Action, ControlState, State, Task};

use aoide_core::entity::EntityUid;

use reqwest::Url;

#[derive(Debug)]
pub enum Intent {
    FetchStatus {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    FetchProgress,
    StartScan {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    StartImport {
        collection_uid: EntityUid,
        root_url: Option<Url>,
    },
    Abort,
    Untrack {
        collection_uid: EntityUid,
        root_url: Url,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> (StateMutation, Option<Action>) {
        log::trace!("Applying intent {:?} on {:?}", self, state);
        match self {
            Self::FetchProgress => {
                state.remote.progress.set_pending_now();
                (
                    StateMutation::Unchanged,
                    Some(Action::dispatch_task(Task::FetchProgress)),
                )
            }
            Self::Abort => (
                StateMutation::Unchanged,
                Some(Action::dispatch_task(Task::Abort)),
            ),
            Self::FetchStatus {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot fetch status while not idle");
                    return (StateMutation::Unchanged, None);
                }
                state.control = ControlState::Busy;
                state.remote.status.set_pending_now();
                (
                    StateMutation::MaybeChanged,
                    Some(Action::dispatch_task(Task::FetchStatus {
                        collection_uid,
                        root_url,
                    })),
                )
            }
            Self::StartScan {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start scan while not idle");
                    return (StateMutation::Unchanged, None);
                }
                state.control = ControlState::Busy;
                state.remote.progress.reset();
                state.remote.status.set_pending_now();
                state.remote.last_scan_outcome.set_pending_now();
                (
                    StateMutation::MaybeChanged,
                    Some(Action::dispatch_task(Task::StartScan {
                        collection_uid,
                        root_url,
                    })),
                )
            }
            Self::StartImport {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot start import while not idle");
                    return (StateMutation::Unchanged, None);
                }
                state.control = ControlState::Busy;
                state.remote.progress.reset();
                state.remote.status.set_pending_now();
                state.remote.last_import_outcome.set_pending_now();
                (
                    StateMutation::MaybeChanged,
                    Some(Action::dispatch_task(Task::StartImport {
                        collection_uid,
                        root_url,
                    })),
                )
            }
            Self::Untrack {
                collection_uid,
                root_url,
            } => {
                if !state.is_idle() {
                    log::warn!("Cannot untrack while not idle");
                    return (StateMutation::Unchanged, None);
                }
                state.control = ControlState::Busy;
                state.remote.progress.reset();
                state.remote.status.set_pending_now();
                state.remote.last_untrack_outcome.set_pending_now();
                (
                    StateMutation::MaybeChanged,
                    Some(Action::dispatch_task(Task::Untrack {
                        collection_uid,
                        root_url,
                    })),
                )
            }
        }
    }
}
