// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::collection::{Collection, EntityUid};

use super::{Action, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    FetchAllKinds,
    FetchFilteredEntities { filter_by_kind: Option<String> },
    ActivateEntity { entity_uid: Option<EntityUid> },
    CreateEntity { new_collection: Collection },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {self:?} on {state:?}");
        match self {
            Self::FetchAllKinds => {
                if let Some(token) = state.remote_view.all_kinds.try_start_pending_now() {
                    let task = Task::FetchAllKinds { token };
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::FetchAllKinds;
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    StateUpdated::unchanged(None)
                }
            }
            Self::FetchFilteredEntities { filter_by_kind } => {
                if let Some(token) = state.remote_view.filtered_entities.try_start_pending_now() {
                    let task = Task::FetchFilteredEntities {
                        token,
                        filter_by_kind,
                    };
                    log::debug!("Dispatching task {task:?}");
                    StateUpdated::maybe_changed(Action::dispatch_task(task))
                } else {
                    let self_reconstructed = Self::FetchFilteredEntities { filter_by_kind };
                    log::warn!("Discarding intent while already pending: {self_reconstructed:?}");
                    StateUpdated::unchanged(None)
                }
            }
            Self::ActivateEntity { entity_uid } => {
                state.set_active_entity_uid(entity_uid);
                StateUpdated::maybe_changed(None)
            }
            Self::CreateEntity { new_collection } => {
                let task = Task::CreateEntity { new_collection };
                log::debug!("Dispatching task {task:?}");
                StateUpdated::unchanged(Action::dispatch_task(task))
            }
        }
    }
}
