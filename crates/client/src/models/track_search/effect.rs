// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Action, FetchResultPageResponse, State, StateUpdated};

#[derive(Debug)]
pub enum Effect {
    FetchResultPageFinished(anyhow::Result<FetchResultPageResponse>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying effect {self:?} on {state:?}");
        match self {
            Self::FetchResultPageFinished(res) => match res {
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
