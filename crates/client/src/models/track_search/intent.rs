// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core_api::track::search::Params;

use crate::prelude::*;

use super::{Action, FetchResultPageRequest, State, StateUpdated, Task};

#[derive(Debug)]
pub enum Intent {
    Reset(Option<Params>),
    FetchResultPage {
        collection_uid: CollectionUid,
        request: FetchResultPageRequest,
    },
}

impl Intent {
    pub fn apply_on(self, state: &mut State) -> StateUpdated {
        log::trace!("Applying intent {self:?} on {state:?}");
        match self {
            Self::Reset(search_params) => {
                if !state.can_reset() {
                    log::warn!("Cannot fetch results: {search_params:?}");
                    return StateUpdated::unchanged(None);
                }
                state.reset(search_params);
                StateUpdated::maybe_changed(None)
            }
            Self::FetchResultPage {
                collection_uid,
                request,
            } => {
                let task = Task::FetchResultPage {
                    collection_uid,
                    request,
                };
                if !state.can_fetch_results() {
                    log::warn!("Cannot fetch results: {task:?}");
                    return StateUpdated::unchanged(None);
                }
                state.set_fetching_results();
                StateUpdated::maybe_changed(Action::dispatch_task(task))
            }
        }
    }
}
