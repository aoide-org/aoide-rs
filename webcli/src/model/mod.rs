// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub(crate) mod effect;
use std::path::PathBuf;

pub(crate) use self::effect::Effect;

pub(crate) mod environment;
pub(crate) use self::environment::Environment;

pub(crate) mod intent;
pub(crate) use self::intent::Intent;

pub(crate) mod state;
pub(crate) use self::state::State;

pub(crate) mod task;
pub(crate) use self::task::Task;

pub(crate) type Action = infect::Action<Effect, Task>;

pub(crate) type Message = infect::Message<Intent, Effect>;
pub(crate) type MessageSender = infect::MessageSender<Intent, Effect>;

pub(crate) type StateUpdated = infect::StateUpdated<Effect, Task>;

impl From<Intent> for Message {
    fn from(intent: Intent) -> Self {
        Self::Intent(intent)
    }
}

impl From<Effect> for Message {
    fn from(effect: Effect) -> Self {
        Self::Effect(effect)
    }
}

#[derive(Debug, Clone)]
pub struct ExportTracksParams {
    pub track_search: aoide_core_api::track::search::Params,
    pub output_file_path: PathBuf,
}

use aoide_core::collection::EntityUid as CollectionUid;

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
