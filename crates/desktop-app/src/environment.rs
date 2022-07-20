// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::Path, sync::Arc};

use aoide_backend_embedded::storage::DatabaseConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::{settings, Settings};

#[allow(missing_debug_implementations)]
pub struct Environment {
    db_config: DatabaseConfig,
    db_gatekeeper: Arc<Gatekeeper>,
    settings_state: Arc<settings::ObservableState>,
}

impl Environment {
    /// Set up the runtime environment.
    ///
    /// The database configuration is created from the initial settings and
    /// immutable. Switching the database connection and settings at runtime
    /// is not supported.
    pub fn commission(initial_settings: Settings) -> anyhow::Result<Self> {
        log::info!("Commissioning runtime environment from settings");
        let db_config = initial_settings.create_database_config()?;
        let db_gatekeeper = aoide_backend_embedded::storage::provision_database(&db_config)?;
        let settings_state = settings::ObservableState::new(initial_settings);
        Ok(Self {
            db_config,
            db_gatekeeper: Arc::new(db_gatekeeper),
            settings_state: Arc::new(settings_state),
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

    #[must_use]
    pub fn db_storage_dir(&self) -> Option<&Path> {
        self.db_config.connection.storage_dir()
    }

    /// Access the database.
    #[must_use]
    pub fn db_gatekeeper(&self) -> &Arc<Gatekeeper> {
        &self.db_gatekeeper
    }

    /// Manage the settings.
    #[must_use]
    pub fn settings_state(&self) -> &Arc<settings::ObservableState> {
        &self.settings_state
    }
}
