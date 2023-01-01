// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_core::track::Entity;

use aoide_core_api::{track::search::Params, Pagination};

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod state;
pub use self::state::{ControlState, State};

pub mod task;
pub use self::task::Task;

pub type Action = crate::action::Action<Effect, Task>;
pub type StateUpdated = crate::state::StateUpdated<Effect, Task>;

#[derive(Debug, Clone)]
pub struct FetchResultPageRequest {
    pub search_params: Params,
    pub pagination: Pagination,
}

#[derive(Debug)]
pub struct FetchResultPageResponse {
    pub entities: Vec<Entity>,
    pub pagination: Pagination,
}
