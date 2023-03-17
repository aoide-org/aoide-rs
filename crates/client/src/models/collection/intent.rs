// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, EntityUid};

use super::{Action, Effect, FetchFilteredEntities, IntentHandled, Model, PendingTask, Task};

#[derive(Debug)]
pub enum Intent {
    FetchAllKinds,
    FetchFilteredEntities(FetchFilteredEntities),
    ActivateEntity { entity_uid: Option<EntityUid> },
    CreateEntity { new_collection: Collection },
}

impl Intent {
    #[must_use]
    pub fn apply_on(self, model: &Model) -> IntentHandled {
        log::trace!("Applying intent {self:?} on {model:?}");
        let next_action = match self {
            Self::FetchAllKinds => {
                if model.remote_view.all_kinds.is_pending() {
                    let self_reconstructed = Self::FetchAllKinds;
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let task = PendingTask::FetchAllKinds;
                let effect = Effect::PendingTaskAccepted { task };
                Action::apply_effect(effect)
            }
            Self::FetchFilteredEntities(FetchFilteredEntities { filter_by_kind }) => {
                if model.remote_view.all_kinds.is_pending() {
                    let self_reconstructed = Self::FetchAllKinds;
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let task =
                    PendingTask::FetchFilteredEntities(FetchFilteredEntities { filter_by_kind });
                let effect = Effect::PendingTaskAccepted { task };
                Action::apply_effect(effect)
            }
            Self::ActivateEntity { entity_uid } => {
                if model.remote_view.all_kinds.is_pending() {
                    let self_reconstructed = Self::ActivateEntity { entity_uid };
                    log::warn!("Discarding intent while still pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::ActiveEntityUidUpdated { entity_uid };
                Action::apply_effect(effect)
            }
            Self::CreateEntity { new_collection } => {
                let task = Task::CreateEntity { new_collection };
                log::debug!("Dispatching task {task:?}");
                Action::spawn_task(task)
            }
        };
        IntentHandled::Accepted(Some(next_action))
    }
}
