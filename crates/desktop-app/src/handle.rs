// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ops::Deref,
    sync::{Arc, Weak},
};

use aoide_backend_embedded::storage::DatabaseConfig;

use crate::Environment;

/// Shared runtime environment handle
///
/// A cheaply `Clone`able and `Send`able handle to a shared runtime environment
/// for invoking operations.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Handle(Arc<Environment>);

impl Handle {
    /// Set up a shared runtime environment
    ///
    /// See also: [`Environment::commission()`]
    pub fn commission(db_config: &DatabaseConfig) -> anyhow::Result<Self> {
        let context = Environment::commission(db_config)?;
        Ok(Self(Arc::new(context)))
    }

    #[must_use]
    pub fn downgrade(&self) -> WeakHandle {
        WeakHandle(Arc::downgrade(&self.0))
    }
}

impl AsRef<Environment> for Handle {
    fn as_ref(&self) -> &Environment {
        &self.0
    }
}

impl Deref for Handle {
    type Target = Environment;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct WeakHandle(Weak<Environment>);

impl WeakHandle {
    #[must_use]
    pub fn upgrade(&self) -> Option<Handle> {
        self.0.upgrade().map(Handle)
    }
}
