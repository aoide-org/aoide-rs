// SPDX-FileCopyrightText: Copyright (C) 2018-2024 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_storage_sqlite::connection::{
    pool::{create_connection_pool, gatekeeper::Gatekeeper, get_pooled_connection},
    Config as ConnectionConfig,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DatabaseSchemaMigrationMode {
    DontTouch,
    #[default]
    ApplyPending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DatabaseConfig {
    pub connection: ConnectionConfig,

    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub migrate_schema: Option<DatabaseSchemaMigrationMode>,
}

pub fn provision_database(config: &DatabaseConfig) -> anyhow::Result<Gatekeeper> {
    let DatabaseConfig {
        connection,
        migrate_schema,
    } = config;

    log::info!(
        "Provisioning SQLite database: {storage}",
        storage = connection.storage,
    );

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    log::info!(
        "Creating connection pool of max. size {max_size}",
        max_size = connection.pool.max_size
    );
    let connection_pool = create_connection_pool(&connection.storage, connection.pool.max_size)?;

    log::info!("Initializing database");
    aoide_repo_sqlite::initialize_database(&mut *get_pooled_connection(&connection_pool)?)?;

    let migrate_schema = (*migrate_schema).unwrap_or_default();
    match migrate_schema {
        DatabaseSchemaMigrationMode::DontTouch => (),
        DatabaseSchemaMigrationMode::ApplyPending => {
            log::info!("Migrating database schema");
            aoide_usecases_sqlite::database::migrate_schema(&mut *get_pooled_connection(
                &connection_pool,
            )?)?;
        }
    }

    let gatekeeper = Gatekeeper::new(connection_pool, config.connection.pool.gatekeeper);

    Ok(gatekeeper)
}
