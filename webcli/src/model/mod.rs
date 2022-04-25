// aoide.org - Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
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

pub(crate) type Action = aoide_client::action::Action<Effect, Task>;

pub(crate) type Message = aoide_client::message::Message<Intent, Effect>;
pub(crate) type MessageSender = aoide_client::messaging::MessageSender<Intent, Effect>;

pub(crate) type StateUpdated = aoide_client::state::StateUpdated<Effect, Task>;

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

///////////////////////////////////////////////////////////////////////
// Tests
///////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests;
