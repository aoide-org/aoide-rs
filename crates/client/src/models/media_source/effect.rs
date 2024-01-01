// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::{EffectApplied, Model, PendingTask, PurgeOrphaned, PurgeUntracked, Task};
use crate::util::roundtrip::PendingToken;

#[derive(Debug)]
pub enum Effect {
    PurgeOrphanedAccepted(PurgeOrphaned),
    PurgeOrphanedFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::source::purge_orphaned::Outcome>,
    },
    PurgeUntrackedAccepted(PurgeUntracked),
    PurgeUntrackedFinished {
        token: PendingToken,
        result: anyhow::Result<aoide_core_api::media::source::purge_untracked::Outcome>,
    },
    ErrorOccurred(anyhow::Error),
}

impl Effect {
    pub fn apply_on(self, model: &mut Model) -> EffectApplied {
        log::trace!("Applying effect {self:?} on {model:?}");
        match self {
            Self::PurgeOrphanedAccepted(purge_orphaned) => {
                debug_assert!(!model.remote_view().last_purge_orphaned_outcome.is_pending());
                model.last_error = None;
                let token = model
                    .remote_view
                    .last_purge_orphaned_outcome
                    .start_pending_now();
                let task = PendingTask::PurgeOrphaned(purge_orphaned);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed_task(task)
            }
            Self::PurgeOrphanedFinished { token, result } => match result {
                Ok(outcome) => {
                    if let Err(outcome) = model
                        .remote_view
                        .last_purge_orphaned_outcome
                        .finish_pending_with_value_now(token, outcome)
                    {
                        let effect_reconstructed = Self::PurgeOrphanedFinished {
                            token,
                            result: Ok(outcome),
                        };
                        log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged();
                    }
                    EffectApplied::maybe_changed()
                }
                Err(err) => {
                    model.remote_view.last_purge_orphaned_outcome.reset();
                    model.last_error = Some(err);
                    EffectApplied::maybe_changed()
                }
            },
            Self::PurgeUntrackedAccepted(purge_untracked) => {
                debug_assert!(!model
                    .remote_view()
                    .last_purge_untracked_outcome
                    .is_pending());
                model.last_error = None;
                let token = model
                    .remote_view
                    .last_purge_untracked_outcome
                    .start_pending_now();
                let task = PendingTask::PurgeUntracked(purge_untracked);
                let task = Task::Pending { token, task };
                EffectApplied::maybe_changed_task(task)
            }
            Self::PurgeUntrackedFinished { token, result } => match result {
                Ok(outcome) => {
                    if let Err(outcome) = model
                        .remote_view
                        .last_purge_untracked_outcome
                        .finish_pending_with_value_now(token, outcome)
                    {
                        let effect_reconstructed = Self::PurgeUntrackedFinished {
                            token,
                            result: Ok(outcome),
                        };
                        log::warn!("Discarding outdated effect: {effect_reconstructed:?}");
                        return EffectApplied::unchanged();
                    }
                    EffectApplied::maybe_changed()
                }
                Err(err) => {
                    model.remote_view.last_purge_orphaned_outcome.reset();
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
