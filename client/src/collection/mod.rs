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

pub mod intent;
pub use self::intent::Intent;

pub mod model;
pub use self::model::{Model, RemoteView};

pub mod task;
pub use self::task::Task;

pub type Action = crate::prelude::Action<Effect, Task>;
pub type ModelUpdate = crate::prelude::mutable::ModelUpdated<Effect, Task>;
