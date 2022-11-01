// SPDX-FileCopyrightText: Copyright (C) 2018-2022 Uwe Klotz <uwedotklotzatgmaildotcom> et al.
// SPDX-License-Identifier: AGPL-3.0-or-later

use aoide_repo_sqlite::MigrationMode;
use serde::{Deserialize, Serialize};

use aoide_storage_sqlite::connection::{
    pool::{create_connection_pool, gatekeeper::Gatekeeper, get_pooled_connection},
    Config as ConnectionConfig,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseSchemaMigrationMode {
    DontTouch,
    ApplyPending,
    ReapplyAll,
}

impl From<DatabaseSchemaMigrationMode> for Option<MigrationMode> {
    fn from(from: DatabaseSchemaMigrationMode) -> Self {
        match from {
            DatabaseSchemaMigrationMode::DontTouch => None,
            DatabaseSchemaMigrationMode::ApplyPending => Some(MigrationMode::ApplyPending),
            DatabaseSchemaMigrationMode::ReapplyAll => Some(MigrationMode::ReapplyAll),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub connection: ConnectionConfig,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub migrate_schema: Option<DatabaseSchemaMigrationMode>,
}

pub fn provision_database(config: &DatabaseConfig) -> anyhow::Result<Gatekeeper> {
    let DatabaseConfig {
        connection,
        migrate_schema,
    } = config;

    log::info!("Provisioning SQLite database: {}", connection.storage,);

    // The maximum size of the pool defines the maximum number of
    // allowed readers while writers require exclusive access.
    log::info!(
        "Creating connection pool of max. size {}",
        connection.pool.max_size
    );
    let connection_pool = create_connection_pool(&connection.storage, connection.pool.max_size)?;

    log::info!("Initializing database");
    aoide_repo_sqlite::initialize_database(&mut *get_pooled_connection(&connection_pool)?)?;

    let migrate_schema = (*migrate_schema).unwrap_or(DatabaseSchemaMigrationMode::ApplyPending);
    let migration_mode: Option<MigrationMode> = migrate_schema.into();
    if let Some(migration_mode) = migration_mode {
        log::info!("Migrating database schema");
        aoide_usecases_sqlite::database::migrate_schema(
            &mut *get_pooled_connection(&connection_pool)?,
            migration_mode,
        )?;
    }

    let gatekeeper = Gatekeeper::new(connection_pool, config.connection.pool.gatekeeper);

    Ok(gatekeeper)
}
