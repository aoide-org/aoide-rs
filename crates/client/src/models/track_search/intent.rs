// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Effect, FetchResultPage, IntentHandled, Model, Reset};

#[derive(Debug)]
pub enum Intent {
    Reset(Reset),
    FetchResultPage(FetchResultPage),
}

impl Intent {
    pub fn handle_on(self, model: &mut Model) -> IntentHandled {
        log::trace!("Applying intent {self:?} on {model:?}");
        match self {
            Self::Reset(reset) => {
                if !model.can_reset() {
                    let self_reconstructed = Self::Reset(reset);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::Reset(reset);
                effect.apply_on(model).into()
            }
            Self::FetchResultPage(fetch_result_page) => {
                if !model.can_fetch_results() {
                    let self_reconstructed = Self::FetchResultPage(fetch_result_page);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::FetchResultPageAccepted(fetch_result_page);
                effect.apply_on(model).into()
            }
        }
    }
}
