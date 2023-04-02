// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{
    ops::Deref,
    sync::{Arc, Weak},
};

use aoide_backend_embedded::storage::DatabaseConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

/// Runtime environment for invoking operations
///
/// Holds the database connection.
#[allow(missing_debug_implementations)]
pub struct Environment {
    db_gatekeeper: Gatekeeper,
}

impl Environment {
    /// Set up the runtime environment
    ///
    /// Modifying the database configuration at runtime is not supported.
    pub fn commission(db_config: &DatabaseConfig) -> anyhow::Result<Self> {
        log::info!("Commissioning context");
        let db_gatekeeper = aoide_backend_embedded::storage::provision_database(db_config)?;
        Ok(Self { db_gatekeeper })
    }

    /// Prepare for shutting down
    ///
    /// Rejects new database requests. Pending requests could still proceed
    /// until finished.
    ///
    /// It is safe to invoke this operation repeatedly.
    pub fn decommission(&self) {
        log::info!("Decommissioning context");
        self.db_gatekeeper.decommission();
    }

    /// The [`Gatekeeper`] for accessing the database
    pub fn db_gatekeeper(&self) -> &Gatekeeper {
        &self.db_gatekeeper
    }
}

impl AsRef<Gatekeeper> for Environment {
    fn as_ref(&self) -> &Gatekeeper {
        self.db_gatekeeper()
    }
}

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
