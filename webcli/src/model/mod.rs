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

pub type Action = aoide_client::prelude::Action<Effect, Task>;
pub type Message = aoide_client::prelude::Message<Intent, Effect>;
pub type MessageSender = aoide_client::prelude::MessageSender<Intent, Effect>;
pub type StateUpdated = aoide_client::prelude::mutable::StateUpdated<Effect, Task>;

#[allow(dead_code)] // unused
pub type MessageReceiver = aoide_client::prelude::MessageReceiver<Intent, Effect>;

#[allow(dead_code)] // unused
pub type MessageChannel = aoide_client::prelude::MessageChannel<Intent, Effect>;

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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
