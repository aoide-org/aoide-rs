// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::sync::Arc;

use aoide_backend_embedded::storage::DatabaseConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

#[allow(missing_debug_implementations)]
pub struct Environment {
    db_config: DatabaseConfig,
    db_gatekeeper: Arc<Gatekeeper>,
}

impl Environment {
    /// Set up the runtime environment.
    ///
    /// Modifying the database configuration at runtime is not supported.
    pub fn commission(db_config: DatabaseConfig) -> anyhow::Result<Self> {
        log::info!("Commissioning runtime environment");
        let db_gatekeeper = aoide_backend_embedded::storage::provision_database(&db_config)?;
        Ok(Self {
            db_config,
            db_gatekeeper: Arc::new(db_gatekeeper),
        })
    }

    /// Prepare for tear down.
    ///
    /// Rejects new database requests. Pending requests could still proceed
    /// until finished.
    pub fn decommission(&self) {
        log::info!("Decommissioning runtime environment");
        self.db_gatekeeper().decommission();
    }

    /// The database configuration.
    #[must_use]
    pub fn db_config(&self) -> &DatabaseConfig {
        &self.db_config
    }

    /// Access the database.
    #[must_use]
    pub fn db_gatekeeper(&self) -> &Arc<Gatekeeper> {
        &self.db_gatekeeper
    }
}
