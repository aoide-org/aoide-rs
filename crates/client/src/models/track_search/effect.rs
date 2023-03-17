// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{Action, EffectApplied, FetchResultPage, FetchResultPageResponse, Model, Reset, Task};

#[derive(Debug)]
pub enum Effect {
    Reset(Reset),
    FetchResultPageAccepted(FetchResultPage),
    FetchResultPageFinished(anyhow::Result<FetchResultPageResponse>),
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, model: &mut Model) -> EffectApplied {
        log::trace!("Applying effect {self:?} on {model:?}");
        match self {
            Self::Reset(Reset { params }) => {
                debug_assert!(model.can_reset());
                model.reset(params);
                EffectApplied::maybe_changed_done()
            }
            Self::FetchResultPageAccepted(fetch_result_page) => {
                debug_assert!(model.can_fetch_results());
                model.set_fetching_results();
                let task = Task::FetchResultPage(fetch_result_page);
                let next_action = Action::spawn_task(task);
                EffectApplied::maybe_changed(Some(next_action))
            }
            Self::FetchResultPageFinished(res) => match res {
                Ok(response) => {
                    model.append_fetched_result_page(response);
                    EffectApplied::maybe_changed_done()
                }
                Err(err) => {
                    EffectApplied::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
                }
            },
            Self::ErrorOccurred(err) => {
                EffectApplied::unchanged(Action::apply_effect(Self::ErrorOccurred(err)))
            }
        }
    }
}
