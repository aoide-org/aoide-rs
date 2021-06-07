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

use crate::models::{active_collection, media_tracker};

pub mod effect;
pub use self::effect::Effect;

pub mod environment;
pub use self::environment::Environment;

pub mod intent;
pub use self::intent::Intent;

pub mod state;
pub use self::state::State;

pub mod task;
pub use self::task::Task;

pub type Action = crate::prelude::Action<Effect, Task>;
pub type Message = crate::prelude::Message<Intent, Effect>;
pub type MessageSender = crate::prelude::MessageSender<Intent, Effect>;
pub type MessageReceiver = crate::prelude::MessageReceiver<Intent, Effect>;
pub type MessageChannel = crate::prelude::MessageChannel<Intent, Effect>;
pub type StateUpdated = crate::prelude::mutable::StateUpdated<Effect, Task>;

impl From<active_collection::Effect> for Action {
    fn from(effect: active_collection::Effect) -> Self {
        Self::ApplyEffect(effect.into())
    }
}

impl From<active_collection::Task> for Action {
    fn from(task: active_collection::Task) -> Self {
        Self::DispatchTask(task.into())
    }
}

impl From<media_tracker::Effect> for Action {
    fn from(effect: media_tracker::Effect) -> Self {
        Self::ApplyEffect(effect.into())
    }
}

impl From<media_tracker::Task> for Action {
    fn from(task: media_tracker::Task) -> Self {
        Self::DispatchTask(task.into())
    }
}

impl From<active_collection::Action> for Action {
    fn from(action: active_collection::Action) -> Self {
        match action {
            active_collection::Action::ApplyEffect(effect) => effect.into(),
            active_collection::Action::DispatchTask(task) => task.into(),
        }
    }
}

impl From<media_tracker::Action> for Action {
    fn from(action: media_tracker::Action) -> Self {
        match action {
            media_tracker::Action::ApplyEffect(effect) => effect.into(),
            media_tracker::Action::DispatchTask(task) => task.into(),
        }
    }
}

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

impl From<active_collection::Intent> for Message {
    fn from(intent: active_collection::Intent) -> Self {
        Self::Intent(intent.into())
    }
}

impl From<active_collection::Effect> for Message {
    fn from(effect: active_collection::Effect) -> Self {
        Self::Effect(effect.into())
    }
}

impl From<media_tracker::Intent> for Message {
    fn from(intent: media_tracker::Intent) -> Self {
        Self::Intent(intent.into())
    }
}

impl From<media_tracker::Effect> for Message {
    fn from(effect: media_tracker::Effect) -> Self {
        Self::Effect(effect.into())
    }
}
