// aoide.org - Copyright (C) 2018-2021 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use aoide_core::track::Entity;

use aoide_core_ext::track::search::Params;

use super::FetchResultPageResponse;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlState {
    Idle,
    FetchingResults,
    Done,
}

impl Default for ControlState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Default)]
pub struct State {
    control_state: ControlState,
    search_params: Option<Params>,
    results: Vec<Entity>,
}

impl State {
    pub fn control_state(&self) -> ControlState {
        self.control_state
    }

    pub fn search_params(&self) -> Option<&Params> {
        self.search_params.as_ref()
    }

    pub fn results(&self) -> &[Entity] {
        &self.results
    }

    pub fn can_reset(&self) -> bool {
        self.control_state != ControlState::FetchingResults
    }

    pub fn can_fetch_results(&self) -> bool {
        self.control_state == ControlState::Idle && self.search_params.is_some()
    }

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
