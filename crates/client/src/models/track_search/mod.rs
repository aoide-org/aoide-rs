// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{collection::EntityUid as CollectionUid, track::Entity};

use aoide_core_api::{track::search::Params, Pagination};

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod task;
pub use self::task::Task;

pub type Action = infect::Action<Effect, Task>;

pub type IntentHandled = infect::IntentHandled<Intent, Effect, Task>;

pub type EffectApplied = infect::EffectApplied<Effect, Task>;

#[derive(Debug, Clone)]
pub struct Reset {
    params: Option<Params>,
}

#[derive(Debug, Clone)]
pub struct FetchResultPage {
    collection_uid: CollectionUid,
    request: FetchResultPageRequest,
}

#[derive(Debug, Clone)]
pub struct FetchResultPageRequest {
    pub params: Params,
    pub pagination: Pagination,
}

#[derive(Debug)]
pub struct FetchResultPageResponse {
    pub entities: Vec<Entity>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ControlState {
    #[default]
    Idle,
    FetchingResults,
    Done,
}

#[derive(Debug, Default)]
pub struct Model {
    control_state: ControlState,
    search_params: Option<Params>,
    results: Vec<Entity>,
}

impl Model {
    #[must_use]
    pub fn control_state(&self) -> ControlState {
        self.control_state
    }

    #[must_use]
    pub fn search_params(&self) -> Option<&Params> {
        self.search_params.as_ref()
    }

    #[must_use]
    pub fn results(&self) -> &[Entity] {
        &self.results
    }

    #[must_use]
    pub fn can_reset(&self) -> bool {
        self.control_state != ControlState::FetchingResults
    }

    #[must_use]
    pub fn can_fetch_results(&self) -> bool {
        self.control_state == ControlState::Idle && self.search_params.is_some()
    }

    #[must_use]
    pub fn search_params_for_fetching_results(&self) -> Option<&Params> {
        if self.can_fetch_results() {
            self.search_params()
        } else {
            None
        }
    }

    pub(super) fn reset(&mut self, search_params: impl Into<Option<Params>>) {
        debug_assert!(self.can_reset());
        self.control_state = ControlState::Idle;
        self.search_params = search_params.into();
        self.results.clear();
    }

    pub(super) fn set_fetching_results(&mut self) {
        debug_assert!(self.can_fetch_results());
        self.control_state = ControlState::FetchingResults;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn append_fetched_result_page(&mut self, response: FetchResultPageResponse) {
        debug_assert!(self.search_params.is_some());
        debug_assert_eq!(self.control_state, ControlState::FetchingResults);
        let FetchResultPageResponse {
            entities,
            pagination,
        } = response;
        debug_assert_eq!(self.results.len(), pagination.mandatory_offset() as usize);
        debug_assert!(entities.len() <= pagination.mandatory_limit() as usize);
        self.control_state = if entities.len() < pagination.mandatory_limit() as usize {
            // Final page
            ControlState::Done
        } else {
            // More results might be available
            ControlState::Idle
        };
        if self.results.is_empty() {
            // First page
            self.results = entities;
        } else {
            // Next page
            let mut entities = entities;
            self.results.append(&mut entities);
        }
    }
}
