// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

mod environment;
pub use self::environment::{Environment, Handle, WeakHandle};

/// File system utilities
pub mod fs;

/// Collection management
pub mod collection;

/// Settings management
pub mod settings;

/// Track management
pub mod track;
