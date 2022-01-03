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

use crate::util::roundtrip::PendingToken;

use super::{Action, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    PurgeOrphanedFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::source::purge_orphaned::Outcome>,
    },
    PurgeUntrackedFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::source::purge_untracked::Outcome>,
    },
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::PurgeOrphanedFinished { token, result } => {
                let next_action = match result {
                    Ok(outcome) => {
                        if let Err(outcome) = state
                            .remote_view
                            .last_purge_orphaned_outcome
                            .finish_pending_with_value_now(token, outcome)
                        {
                            let effect_reconstructed = Self::PurgeOrphanedFinished {
                                token,
                                result: Ok(outcome),
                            };
                            log::warn!("Discarding outdated effect: {:?}", effect_reconstructed);
                            return StateUpdated::unchanged(None);
                        }
                        None
                    }
                    Err(err) => {
                        state.remote_view.last_purge_orphaned_outcome.reset();
                        Some(Action::apply_effect(Self::ErrorOccurred(err)))
                    }
                };
                StateUpdated::maybe_changed(next_action)
            }
            Self::PurgeUntrackedFinished { token, result } => {
                let next_action = match result {
                    Ok(outcome) => {
                        if let Err(outcome) = state
                            .remote_view
                            .last_purge_untracked_outcome
                            .finish_pending_with_value_now(token, outcome)
                        {
                            let effect_reconstructed = Self::PurgeUntrackedFinished {
                                token,
                                result: Ok(outcome),
                            };
                            log::warn!("Discarding outdated effect: {:?}", effect_reconstructed);
                            return StateUpdated::unchanged(None);
                        }
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
