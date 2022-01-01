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

use super::{Action, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    PurgeOrphanedFinished(anyhow::Result<aoide_core_api::media::source::purge_orphaned::Outcome>),
    PurgeUntrackedFinished(anyhow::Result<aoide_core_api::media::source::purge_untracked::Outcome>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::PurgeOrphanedFinished(res) => {
                if !state.remote_view.last_purge_orphaned_outcome.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::PurgeOrphanedFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_purge_orphaned_outcome =
                            RemoteData::ready_now(outcome);
                        None
                    }
                    Err(err) => {
                        state.remote_view.last_purge_orphaned_outcome.reset();
                        Some(Action::apply_effect(Self::ErrorOccurred(err)))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::PurgeUntrackedFinished(res) => {
                if !state.remote_view.last_purge_untracked_outcome.is_pending() {
                    log::warn!(
                        "Discarding effect while not pending: {:?}",
                        Self::PurgeUntrackedFinished(res)
                    );
                    return StateUpdated::unchanged(None);
                }
                let next_action = match res {
                    Ok(outcome) => {
                        state.remote_view.last_purge_untracked_outcome =
                            RemoteData::ready_now(outcome);
                        None
                    }
                    Err(err) => {
                        state.remote_view.last_purge_orphaned_outcome.reset();
                        Some(Action::apply_effect(Self::ErrorOccurred(err)))
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
