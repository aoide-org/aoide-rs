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

use super::{Action, FetchResultPageResponse, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    ResultPageFetched(anyhow::Result<FetchResultPageResponse>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {:?} on {:?}", self, state);
        match self {
            Self::ResultPageFetched(res) => match res {
                Ok(response) => {
                    state.append_fetched_result_page(response);
                    StateUpdated::maybe_changed(None)
                }
                Err(err) => StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err))),
            },
            Self::ErrorOccurred(err) => {
                StateUpdated::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
            }
        }
    }
}
