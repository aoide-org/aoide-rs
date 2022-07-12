// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod intent;
pub use self::intent::Intent;

pub mod effect;
pub use self::effect::Effect;

pub mod state;
pub use self::state::{RemoteView, State};

pub mod task;
pub use self::task::Task;

pub type Action = crate::action::Action<Effect, Task>;
pub type StateUpdated = crate::state::StateUpdated<Effect, Task>;

use aoide_core::collection::EntityUid as CollectionUid;
