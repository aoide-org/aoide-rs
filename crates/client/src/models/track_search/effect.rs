// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{EffectApplied, FetchResultPage, FetchResultPageResponse, Model, Reset, Task};

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
                EffectApplied::maybe_changed()
            }
            Self::FetchResultPageAccepted(fetch_result_page) => {
                debug_assert!(model.can_fetch_results());
                model.set_fetching_results();
                let task = Task::FetchResultPage(fetch_result_page);
                EffectApplied::maybe_changed_task(task)
            }
            Self::FetchResultPageFinished(res) => match res {
                Ok(response) => {
                    model.append_fetched_result_page(response);
                    EffectApplied::maybe_changed()
                }
                Err(err) => {
                    model.last_error = Some(err);
                    EffectApplied::maybe_changed()
                }
            },
            Self::ErrorOccurred(err) => {
                model.last_error = Some(err);
                EffectApplied::maybe_changed()
            }
        }
    }
}
