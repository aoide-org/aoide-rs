// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, EntityUid};

use super::{
    Effect, EffectApplied, FetchFilteredEntities, IntentHandled, Model, PendingTask, Task,
};

#[derive(Debug)]
pub enum Intent {
    FetchAllKinds,
    FetchFilteredEntities(FetchFilteredEntities),
    ActivateEntity { entity_uid: Option<EntityUid> },
    CreateEntity { new_collection: Collection },
}

impl Intent {
    #[must_use]
    pub fn handle_on(self, model: &mut Model) -> IntentHandled {
        log::trace!("Applying intent {self:?} on {model:?}");
        match self {
            Self::FetchAllKinds => {
                if model.remote_view.is_pending() {
                    let self_reconstructed = Self::FetchAllKinds;
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                model.last_error = None;
                let task = PendingTask::FetchAllKinds;
                let token = model.remote_view.all_kinds.start_pending_now();
                let task = Task::Pending { token, task };
                IntentHandled::Accepted(EffectApplied::maybe_changed(task))
            }
            Self::FetchFilteredEntities(fetch_filtered_entities) => {
                if model.remote_view.is_pending() {
                    let self_reconstructed = Self::FetchFilteredEntities(fetch_filtered_entities);
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let FetchFilteredEntities { filter_by_kind } = fetch_filtered_entities;
                model.last_error = None;
                let task =
                    PendingTask::FetchFilteredEntities(FetchFilteredEntities { filter_by_kind });
                let token = model.remote_view.filtered_entities.start_pending_now();
                let task = Task::Pending { token, task };
                IntentHandled::Accepted(EffectApplied::maybe_changed(task))
            }
            Self::ActivateEntity { entity_uid } => {
                if model.remote_view.is_pending() {
                    let self_reconstructed = Self::ActivateEntity { entity_uid };
                    log::warn!("Discarding intent while still pending: {self_reconstructed:?}");
                    return IntentHandled::Rejected(self_reconstructed);
                }
                let effect = Effect::ActiveEntityUidUpdated { entity_uid };
                effect.apply_on(model).into()
            }
            Self::CreateEntity { new_collection } => {
                let task = Task::CreateEntity { new_collection };
                EffectApplied::unchanged(task).into()
            }
        }
    }
}
