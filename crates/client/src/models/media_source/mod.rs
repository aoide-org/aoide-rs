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

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod state;
pub use self::state::{RemoteView, State};

pub mod task;
pub use self::task::Task;

#[cfg(feature = "with-reqwest")]
mod webtask;

pub type Action = crate::action::Action<Effect, Task>;
pub type StateUpdated = crate::state::StateUpdated<Effect, Task>;