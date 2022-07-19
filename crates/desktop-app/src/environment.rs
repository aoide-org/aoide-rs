// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::{path::Path, sync::Arc};

use aoide_backend_embedded::storage::DatabaseConfig;
use aoide_storage_sqlite::connection::pool::gatekeeper::Gatekeeper;

use crate::Settings;

#[allow(missing_debug_implementations)]
pub struct Environment {
    db_config: DatabaseConfig,
    db_gatekeeper: Arc<Gatekeeper>,
}

impl Environment {
    pub fn restore_from_settings(settings: &Settings) -> anyhow::Result<Self> {
        log::info!("Restoring environment from settings");
        let db_config = settings.create_database_config()?;
        let db_gatekeeper = Arc::new(aoide_backend_embedded::storage::provision_database(
            &db_config,
        )?);
        Ok(Self {
            db_config,
            db_gatekeeper,
        })
    }

    #[must_use]
    pub fn storage_dir(&self) -> Option<&Path> {
        self.db_config.connection.storage_dir()
    }

    #[must_use]
    pub fn db_gatekeeper(&self) -> &Arc<Gatekeeper> {
        &self.db_gatekeeper
    }

    #[must_use]
    pub fn db_gatekeeper_owned(&self) -> Arc<Gatekeeper> {
        Arc::clone(self.db_gatekeeper())
    }

    pub fn abort_current_db_task(&self) {
        self.db_gatekeeper.abort_current_task();
    }
}
