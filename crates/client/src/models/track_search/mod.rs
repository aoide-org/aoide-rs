// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::{track::Entity, CollectionUid};
use aoide_core_api::{track::search::Params, Pagination};
use infect::ModelChanged;

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod task;
pub use self::task::Task;

pub type IntentRejected = Intent;
pub type IntentHandled = infect::IntentHandled<IntentRejected, Effect, Task, ModelChanged>;
pub type EffectApplied = infect::EffectApplied<Effect, Task, ModelChanged>;

#[derive(Debug, Clone)]
pub struct Reset {
    pub params: Option<Params>,
}

#[derive(Debug, Clone)]
pub struct FetchResultPage {
    pub collection_uid: CollectionUid,
    pub request: FetchResultPageRequest,
}

#[derive(Debug, Clone)]
pub struct FetchResultPageRequest {
    pub params: Params,
    pub encode_gigtags: bool,
    pub pagination: Pagination,
}

#[derive(Debug)]
pub struct FetchResultPageResponse {
    pub entities: Vec<Entity>,
    pub pagination: Pagination,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum State {
    #[default]
    Idle,
    FetchingResults,
    Done,
}

#[derive(Debug, Default)]
pub struct Model {
    state: State,
    search_params: Option<Params>,
    results: Vec<Entity>,
    last_error: Option<anyhow::Error>,
}

impl Model {
    #[must_use]
    pub fn state(&self) -> State {
        self.state
    }

    #[must_use]
    pub fn last_error(&self) -> Option<&anyhow::Error> {
        self.last_error.as_ref()
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
        self.state != State::FetchingResults
    }

    #[must_use]
    pub fn can_fetch_results(&self) -> bool {
        self.state == State::Idle && self.search_params.is_some()
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
        self.state = State::Idle;
        self.search_params = search_params.into();
        self.results.clear();
        self.last_error = None;
    }

    pub(super) fn set_fetching_results(&mut self) {
        debug_assert!(self.can_fetch_results());
        self.state = State::FetchingResults;
    }

    #[allow(clippy::cast_possible_truncation)]
    pub(super) fn append_fetched_result_page(&mut self, response: FetchResultPageResponse) {
        debug_assert!(self.search_params.is_some());
        debug_assert_eq!(self.state, State::FetchingResults);
        let FetchResultPageResponse {
            entities,
            pagination,
        } = response;
        debug_assert_eq!(self.results.len(), pagination.mandatory_offset() as usize);
        debug_assert!(entities.len() <= pagination.mandatory_limit() as usize);
        self.state = if entities.len() < pagination.mandatory_limit() as usize {
            // Final page
            State::Done
        } else {
            // More results might be available
            State::Idle
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
