// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use serde::{Deserialize, Serialize};

use aoide_storage_sqlite::connection::{
    pool::{create_connection_pool, gatekeeper::Gatekeeper, get_pooled_connection},
    Config as ConnectionConfig,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection: ConnectionConfig,
    pub migrate_schema: bool,
}

pub fn provision_database(config: &DatabaseConfig) -> anyhow::Result<Gatekeeper> {
    log::info!(
        "Provisioning SQLite database: {}",
        config.connection.storage,
    );

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    log::info!(
        "Creating connection pool of max. size {}",
        config.connection.pool.max_size
    );
    let connection_pool =
        create_connection_pool(&config.connection.storage, config.connection.pool.max_size)?;

    log::info!("Initializing database");
    aoide_repo_sqlite::initialize_database(&*get_pooled_connection(&connection_pool)?)?;

    if config.migrate_schema {
        log::info!("Migrating database schema");
        aoide_usecases_sqlite::database::migrate_schema(&*get_pooled_connection(
            &connection_pool,
        )?)?;
    }

    let gatekeeper = Gatekeeper::new(connection_pool, config.connection.pool.gatekeeper);

    Ok(gatekeeper)
}
