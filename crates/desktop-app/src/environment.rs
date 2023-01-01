// SPDX-FileCopyrightText: Copyright (C) 2018-2023 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::{Arc, Weak};

use aoide_backend_embedded::storage::DatabaseConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

#[allow(missing_debug_implementations)]
pub struct Environment {
    db_config: DatabaseConfig,
    handle: Handle,
}

impl Environment {
    /// Set up the runtime environment.
    ///
    /// Modifying the database configuration at runtime is not supported.
    pub fn commission(db_config: DatabaseConfig) -> anyhow::Result<Self> {
        log::info!("Commissioning runtime environment");
        let db_gatekeeper = aoide_backend_embedded::storage::provision_database(&db_config)?;
        let handle = Handle(Arc::new(db_gatekeeper));
        Ok(Self { db_config, handle })
    }

    /// Prepare for tear down.
    ///
    /// Rejects new database requests. Pending requests could still proceed
    /// until finished.
    pub fn decommission(&self) {
        log::info!("Decommissioning runtime environment");
        self.handle.db_gatekeeper().decommission();
    }

    /// The database configuration.
    #[must_use]
    pub fn db_config(&self) -> &DatabaseConfig {
        &self.db_config
    }

    /// Handle for invoking operations.
    #[must_use]
    pub fn handle(&self) -> &Handle {
        &self.handle
    }
}

/// A cheaply `Clone`able and `Send`able handle for invoking operations.
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Handle(Arc<Gatekeeper>);

impl Handle {
    #[must_use]
    pub fn downgrade(&self) -> WeakHandle {
        WeakHandle(Arc::downgrade(&self.0))
    }

    #[must_use]
    pub fn db_gatekeeper(&self) -> &Gatekeeper {
        &self.0
    }
}

#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct WeakHandle(Weak<Gatekeeper>);

impl WeakHandle {
    #[must_use]
    pub fn upgrade(&self) -> Option<Handle> {
        self.0.upgrade().map(Handle)
    }
}
