// SPDX-FileCopyrightText: Copyright (C) 2018-2025 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

pub mod channel;
pub use self::channel::*;

pub mod sample;
pub use self::sample::*;

pub mod signal;
pub use aoide_core::audio::{DurationMs, PositionMs};

pub use self::signal::*;
