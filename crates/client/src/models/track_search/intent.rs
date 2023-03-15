// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Action, Effect, FetchResultPage, IntentHandled, Model, Reset};

#[derive(Debug)]
pub enum Intent {
    Reset(Reset),
    FetchResultPage(FetchResultPage),
}

impl Intent {
    pub fn apply_on(self, model: &mut Model) -> IntentHandled {
        log::trace!("Applying intent {self:?} on {model:?}");
        let next_action = match self {
            Self::Reset(reset) => {
                if !model.can_reset() {
                    let self_reconstructed = Self::Reset(reset);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::Reset(reset);
                Action::apply_effect(effect)
            }
            Self::FetchResultPage(fetch_result_page) => {
                if !model.can_fetch_results() {
                    let self_reconstructed = Self::FetchResultPage(fetch_result_page);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::FetchResultPageAccepted(fetch_result_page);
                Action::apply_effect(effect)
            }
        };
        infect::IntentHandled::Accepted(Some(next_action))
    }
}
