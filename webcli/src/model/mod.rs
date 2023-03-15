// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use infect::Model as ClientModel;

use aoide_client::models::{collection, media_source, media_tracker};

pub(crate) mod effect;
use std::path::PathBuf;

pub(crate) use self::effect::Effect;

pub(crate) mod environment;
pub(crate) use self::environment::Environment;

pub(crate) mod intent;
pub(crate) use self::intent::Intent;

pub(crate) mod task;
pub(crate) use self::task::Task;

pub(crate) type Action = infect::Action<Effect, Task>;

pub(crate) type Message = infect::Message<Intent, Effect>;

pub(crate) type IntentHandled = infect::IntentHandled<Intent, Effect, Task>;

pub(crate) type EffectApplied = infect::EffectApplied<Effect, Task>;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControlState {
    #[default]
    Running,
    Terminating,
}

#[derive(Debug, Default)]
pub struct Model {
    pub(super) last_errors: Vec<anyhow::Error>,
    pub(super) control_state: ControlState,
    pub active_collection: collection::Model,
    pub media_sources: media_source::Model,
    pub media_tracker: media_tracker::Model,
}

impl Model {
    pub fn last_errors(&self) -> &[anyhow::Error] {
        &self.last_errors
    }

    pub fn is_pending(&self) -> bool {
        self.active_collection.remote_view().is_pending()
            || self.media_sources.remote_view().is_pending()
            || self.media_tracker.remote_view().is_pending()
    }

    pub fn is_terminating(&self) -> bool {
        self.control_state == ControlState::Terminating
    }

    pub fn abort_pending_request_action(&self) -> Option<Action> {
        self.is_pending()
            .then(|| Action::apply_effect(Effect::AbortPendingRequest(None)))
    }
}

impl ClientModel for Model {
    type Intent = Intent;
    type Effect = Effect;
    type Task = Task;

    fn handle_intent(&self, intent: Self::Intent) -> IntentHandled {
        log::debug!("Handling {intent:?} on state {self:?}");
        intent.apply_on(self)
    }

    fn apply_effect(&mut self, effect: Self::Effect) -> EffectApplied {
        log::debug!("Updating state {self:?} with {effect:?}");
        effect.apply_on(self)
    }
}

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
